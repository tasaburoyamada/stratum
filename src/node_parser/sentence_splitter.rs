use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;

use crate::core::config::SplitterConfig;

use tiktoken_rs::{cl100k_base, CoreBPE};
use std::sync::Arc;

#[derive(Clone)]
pub struct SentenceSplitter {
    pub config: SplitterConfig,
    pub secondary_chunking_regex: Regex,
    pub bpe: Arc<CoreBPE>,
}

impl std::fmt::Debug for SentenceSplitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SentenceSplitter")
            .field("config", &self.config)
            .finish()
    }
}

impl Default for SentenceSplitter {
    fn default() -> Self {
        Self::new(SplitterConfig::default())
    }
}

impl SentenceSplitter {
    pub fn new(config: SplitterConfig) -> Self {
        Self {
            config,
            secondary_chunking_regex: Regex::new(r"[^,.;。？！]+[,.;。？！]?|[,.;。？！]").unwrap(),
            bpe: Arc::new(cl100k_base().unwrap()),
        }
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    fn split(&self, text: &str, chunk_size: usize) -> Vec<String> {
        if self.count_tokens(text) <= chunk_size {
            return vec![text.to_string()];
        }

        // 1. Paragraph split
        let paragraphs: Vec<&str> = text.split(&self.config.paragraph_separator).collect();
        if paragraphs.len() > 1 {
            return self.split_recursive(&paragraphs, chunk_size);
        }

        // 2. Secondary regex split (Sentences/Phrases)
        let sentences: Vec<&str> = self.secondary_chunking_regex.find_iter(text).map(|m| m.as_str()).collect();
        if sentences.len() > 1 {
            return self.split_recursive(&sentences, chunk_size);
        }

        // 3. Word split
        let words: Vec<&str> = text.split(&self.config.separator).collect();
        self.split_recursive(&words, chunk_size)
    }

    fn split_recursive(&self, items: &[&str], chunk_size: usize) -> Vec<String> {
        let mut splits = Vec::new();
        for item in items {
            if self.count_tokens(item) <= chunk_size {
                splits.push(item.to_string());
            } else {
                splits.extend(self.split(item, chunk_size));
            }
        }
        splits
    }

    fn merge(&self, splits: Vec<String>, chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut cur_chunk: Vec<String> = Vec::new();
        let mut cur_len = 0;

        for split in splits {
            let split_len = self.count_tokens(&split);
            
            if cur_len + split_len > chunk_size && !cur_chunk.is_empty() {
                chunks.push(cur_chunk.join(""));
                
                // Handle overlap
                let mut overlap_chunk = Vec::new();
                let mut overlap_len = 0;
                for s in cur_chunk.iter().rev() {
                    let s_len = self.count_tokens(s);
                    if overlap_len + s_len <= chunk_overlap {
                        overlap_chunk.insert(0, s.clone());
                        overlap_len += s_len;
                    } else {
                        break;
                    }
                }
                cur_chunk = overlap_chunk;
                cur_len = overlap_len;
            }
            
            cur_len += split_len;
            cur_chunk.push(split);
        }

        if !cur_chunk.is_empty() {
            chunks.push(cur_chunk.join(""));
        }

        chunks
    }
}

#[async_trait]
impl Transformation for SentenceSplitter {
    async fn transform(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        let mut all_new_nodes = Vec::new();

        for node in nodes {
            if let crate::core::schema::NodeContent::Text(text) = &node.content {
                // 1. Calculate effective chunk size
                let metadata_str = node.metadata_to_str();
                let metadata_len = self.count_tokens(&metadata_str);
                
                if metadata_len >= self.config.chunk_size {
                    return Err(anyhow::anyhow!(
                        "Metadata length ({}) exceeds or equals chunk_size ({}). Cannot split node {}.", 
                        metadata_len, self.config.chunk_size, node.id_
                    ));
                }

                let effective_chunk_size = self.config.chunk_size - metadata_len;

                let splits = self.split(text, effective_chunk_size);
                let chunks = self.merge(splits, effective_chunk_size, self.config.chunk_overlap);

                let mut document_nodes = Vec::new();
                for chunk in chunks {
                    let mut new_node = Node::new_text(chunk.clone());
                    new_node.metadata = node.metadata.clone();
                    
                    // Cache tokens in the node (convert from usize to u32 for efficiency if needed, but keeping as is for now)
                    let tokens = self.bpe.encode_with_special_tokens(&chunk);
                    new_node.tokens = Some(tokens.into_iter().map(|t| t as u32).collect());

                    // Python版の振る舞いを模倣: 親ドキュメントへの参照を保持
                    let mut rels = std::collections::HashMap::new();
                    rels.insert(crate::core::schema::NodeRelationship::Source, vec![crate::core::schema::RelatedNodeInfo {
                        node_id: node.id_.clone(),
                        node_type: Some(crate::core::schema::NodeType::Document),
                        metadata: node.metadata.clone(),
                        hash: Some(node.hash()),
                    }]);
                    new_node.relationships = rels;
                    document_nodes.push(new_node);
                }

                // Link Next/Prev relationships
                for i in 0..document_nodes.len() {
                    if i > 0 {
                        let prev_id = document_nodes[i-1].id_.clone();
                        document_nodes[i].relationships.entry(crate::core::schema::NodeRelationship::Previous)
                            .or_insert_with(Vec::new)
                            .push(crate::core::schema::RelatedNodeInfo {
                                node_id: prev_id,
                                node_type: Some(crate::core::schema::NodeType::Text),
                                metadata: std::collections::HashMap::new(),
                                hash: None,
                            });
                    }
                    if i < document_nodes.len() - 1 {
                        let next_id = document_nodes[i+1].id_.clone();
                        document_nodes[i].relationships.entry(crate::core::schema::NodeRelationship::Next)
                            .or_insert_with(Vec::new)
                            .push(crate::core::schema::RelatedNodeInfo {
                                node_id: next_id,
                                node_type: Some(crate::core::schema::NodeType::Text),
                                metadata: std::collections::HashMap::new(),
                                hash: None,
                            });
                    }
                }

                all_new_nodes.extend(document_nodes);
            } else {
                all_new_nodes.push(node);
            }
        }

        Ok(all_new_nodes)
    }

    fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(format!("{:?}", self).as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}
