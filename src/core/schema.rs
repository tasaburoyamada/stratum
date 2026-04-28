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
    Image(Vec<u8>), // Placeholder for actual image data handling
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
    pub fn new_text(text: String) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(text.as_bytes());
        let id_ = hasher.finalize().to_hex().to_string();

        Self {
            id_,
            embedding: None,
            tokens: None,
            metadata: HashMap::new(),
            excluded_embed_metadata_keys: Vec::new(),
            excluded_llm_metadata_keys: Vec::new(),
            relationships: HashMap::new(),
            content: NodeContent::text(text),
            metadata_template: "{key}: {value}".to_string(),
            metadata_separator: "\n".to_string(),
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        // Content-based hashing for determinism
        match &self.content {
            NodeContent::Text(t) => hasher.update(t.as_bytes()),
            NodeContent::Binary(b) | NodeContent::Image(b) => hasher.update(b),
        };
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
