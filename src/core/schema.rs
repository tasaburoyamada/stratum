use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use half::f16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeRelationship {
    Source,
    Previous,
    Next,
    Parent,
    Child,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Text,
    Image,
    Document,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedNodeInfo {
    pub node_id: String,
    pub node_type: Option<NodeType>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    Text(String),
    Binary(Vec<u8>),
    Image(Vec<u8>),
    Purged, // Content has been moved to tokens/embeddings and string is freed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id_: String,
    pub embedding: Option<Vec<f16>>,
    pub tokens: Option<Vec<u32>>, // AI-native token representation
    pub metadata: HashMap<String, serde_json::Value>,
    pub excluded_embed_metadata_keys: Vec<String>,
    pub excluded_llm_metadata_keys: Vec<String>,
    pub relationships: HashMap<NodeRelationship, Vec<RelatedNodeInfo>>,
    pub content: NodeContent,
    pub metadata_template: String,
    pub metadata_separator: String,
}

impl Node {
    /// Discard the original string content to save memory. 
    /// Should only be called after tokens or embeddings are generated.
    pub fn purge_text(&mut self) {
        if let NodeContent::Text(_) = self.content {
            self.content = NodeContent::Purged;
        }
    }

    pub fn get_content(&self, bpe: Option<&tiktoken_rs::CoreBPE>) -> Result<String, anyhow::Error> {
        match &self.content {
            NodeContent::Text(t) => Ok(t.clone()),
            NodeContent::Binary(b) => Ok(format!("<Binary data: {} bytes>", b.len())),
            NodeContent::Image(_) => Ok("<Image data>".to_string()),
            NodeContent::Purged => {
                if let (Some(tokens), Some(bpe)) = (&self.tokens, bpe) {
                    Ok(bpe.decode(tokens.clone()).map_err(|e| anyhow::anyhow!("BPE decode error: {}", e))?)
                } else {
                    Err(anyhow::anyhow!("Node content is purged and no BPE provided for decoding"))
                }
            }
        }
    }

    pub fn new_text(text: String) -> Self {
        let mut node = Self {
            id_: String::new(), // Temporary
            embedding: None,
            tokens: None,
            metadata: HashMap::new(),
            excluded_embed_metadata_keys: Vec::new(),
            excluded_llm_metadata_keys: Vec::new(),
            relationships: HashMap::new(),
            content: NodeContent::text(text),
            metadata_template: "{key}: {value}".to_string(),
            metadata_separator: "\n".to_string(),
        };
        node.id_ = node.hash();
        node
    }

    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        
        // 1. Hash Content
        match &self.content {
            NodeContent::Text(t) => {
                hasher.update(t.as_bytes());
            }
            NodeContent::Binary(b) | NodeContent::Image(b) => {
                hasher.update(b);
            }
            NodeContent::Purged => {
                // For purged nodes, we rely on the pre-calculated ID 
                // but since we want to return a String, we just return id_
                return self.id_.clone();
            }
        }

        // 2. Hash Metadata (Deterministic JSON representation)
        if let Ok(meta_json) = serde_json::to_string(&self.metadata) {
            hasher.update(meta_json.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    pub fn metadata_to_str(&self) -> String {
        let mut metadata_lines = Vec::new();
        for (key, value) in &self.metadata {
            if self.excluded_embed_metadata_keys.contains(key) {
                continue;
            }
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            let line = self.metadata_template
                .replace("{key}", key)
                .replace("{value}", &val_str);
            metadata_lines.push(line);
        }
        metadata_lines.join(&self.metadata_separator)
    }
}

impl NodeContent {
    pub fn text(t: String) -> Self {
        NodeContent::Text(t)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWithScore {
    pub node: Node,
    pub score: Option<f32>,
}
