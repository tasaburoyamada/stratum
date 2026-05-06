use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use crate::embeddings::base::Embedding;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

pub struct SemanticSplitter {
    pub embed_model: Arc<dyn Embedding>,
    pub buffer_size: usize,
    pub breakpoint_percentile_threshold: f32,
    pub sentence_regex: Regex,
}

impl std::fmt::Debug for SemanticSplitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticSplitter")
            .field("buffer_size", &self.buffer_size)
            .field("breakpoint_percentile_threshold", &self.breakpoint_percentile_threshold)
            .finish()
    }
}

impl SemanticSplitter {
    pub fn new(embed_model: Arc<dyn Embedding>) -> Self {
        Self {
            embed_model,
            buffer_size: 1,
            breakpoint_percentile_threshold: 95.0,
            sentence_regex: Regex::new(r"[^.!?。！？]+[.!?。！？]?").unwrap(),
        }
    }

    async fn get_sentence_embeddings(&self, sentences: &[String]) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.embed_model.get_text_embedding_batch(sentences.to_vec(), 32).await?;
        Ok(embeddings.into_iter().map(|e| e.iter().map(|&x| f32::from(x)).collect()).collect())
    }

    fn calculate_distances(&self, embeddings: &[Vec<f32>]) -> Vec<f32> {
        let mut distances = Vec::new();
        if embeddings.len() < 2 {
            return distances;
        }

        for i in 0..embeddings.len() - 1 {
            let dist = 1.0 - cosine_similarity_f32(&embeddings[i], &embeddings[i+1]);
            distances.push(dist);
        }
        distances
    }
}

fn cosine_similarity_f32(v1: &[f32], v2: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_v1 = 0.0;
    let mut norm_v2 = 0.0;
    for i in 0..v1.len() {
        dot_product += v1[i] * v2[i];
        norm_v1 += v1[i] * v1[i];
        norm_v2 += v2[i] * v2[i];
    }
    if norm_v1 <= 0.0 || norm_v2 <= 0.0 { return 0.0; }
    dot_product / (norm_v1.sqrt() * norm_v2.sqrt())
}

#[async_trait]
impl Transformation for SemanticSplitter {
    async fn transform(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        let mut all_new_nodes = Vec::new();

        for node in nodes {
            if let crate::core::schema::NodeContent::Text(text) = &node.content {
                let sentences: Vec<String> = self.sentence_regex.find_iter(text)
                    .map(|m| m.as_str().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if sentences.len() <= 1 {
                    all_new_nodes.push(node);
                    continue;
                }

                let embeddings = self.get_sentence_embeddings(&sentences).await?;
                let distances = self.calculate_distances(&embeddings);

                let mut sorted_distances = distances.clone();
                sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let threshold_idx = (sorted_distances.len() as f32 * self.breakpoint_percentile_threshold / 100.0) as usize;
                let threshold = sorted_distances.get(threshold_idx.min(sorted_distances.len() - 1)).cloned().unwrap_or(0.5);

                let mut chunks = Vec::new();
                let mut current_chunk = Vec::new();

                for (i, sentence) in sentences.into_iter().enumerate() {
                    current_chunk.push(sentence);
                    if i < distances.len() && distances[i] > threshold {
                        chunks.push(current_chunk.join(" "));
                        current_chunk = Vec::new();
                    }
                }
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.join(" "));
                }

                for chunk in chunks {
                    let mut new_node = Node::new_text(chunk);
                    new_node.metadata = node.metadata.clone();
                    all_new_nodes.push(new_node);
                }
            } else {
                all_new_nodes.push(node);
            }
        }

        Ok(all_new_nodes)
    }

    fn hash(&self) -> String {
        format!("semantic_splitter_{}", self.breakpoint_percentile_threshold)
    }
}
