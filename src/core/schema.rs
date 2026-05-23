use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use half::f16;
use std::collections::BTreeMap;

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
    pub metadata: TypedMetadata,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    Text(String),
    Binary(Vec<u8>),
    Image(Vec<u8>),
    Purged, // Content has been moved to tokens/embeddings and string is freed
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypedMetadata {
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub genre: Option<String>,
    pub confidence: Option<f32>, // HV-CAD: Reliability of the information (0.0 - 1.0)
    pub importance: Option<f32>, // HV-CAD: Strategic priority (0.0 - 1.0)
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id_: String,
    pub embedding: Option<Vec<f16>>,
    pub tokens: Option<Vec<u32>>, // AI-native token representation
    pub metadata: TypedMetadata,  // Structured metadata for redb/ACID persistence
    pub excluded_embed_metadata_keys: Vec<String>,
    pub excluded_llm_metadata_keys: Vec<String>,
    pub relationships: HashMap<NodeRelationship, Vec<RelatedNodeInfo>>,
    pub content: NodeContent,
    pub metadata_template: String,
    pub metadata_separator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataContext {
    Embedding,
    Llm,
    General,
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
            id_: String::new(), 
            embedding: None,
            tokens: None,
            metadata: TypedMetadata::default(),
            excluded_embed_metadata_keys: Vec::new(),
            excluded_llm_metadata_keys: Vec::new(),
            relationships: HashMap::new(),
            content: NodeContent::text(text),
            metadata_template: "{key}: {value}".to_string(),
            metadata_separator: "\n".to_string(),
        };
        node.id_ = node.calculate_content_hash();
        node
    }

    /// Calculate hash based purely on content for deterministic identity.
    /// If content is Purged, it returns the current ID (which was derived from original content).
    pub fn calculate_content_hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        match &self.content {
            NodeContent::Text(t) => {
                hasher.update(t.as_bytes());
            }
            NodeContent::Binary(b) | NodeContent::Image(b) => {
                hasher.update(b);
            }
            NodeContent::Purged => {
                // If purged, we cannot re-calculate hash from content.
                // We return the original ID to maintain identity.
                return self.id_.clone();
            }
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Calculate full hash including metadata for strict versioning.
    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        
        // 1. Hash Content Identity
        hasher.update(self.id_.as_bytes());

        // 2. Hash Metadata (Deterministic via BTreeMap sorting)
        let mut deterministic_meta = BTreeMap::new();
        deterministic_meta.insert("url", serde_json::json!(self.metadata.url));
        deterministic_meta.insert("file_path", serde_json::json!(self.metadata.file_path));
        deterministic_meta.insert("timestamp", serde_json::json!(self.metadata.timestamp));
        deterministic_meta.insert("genre", serde_json::json!(self.metadata.genre));
        deterministic_meta.insert("confidence", serde_json::json!(self.metadata.confidence));
        deterministic_meta.insert("importance", serde_json::json!(self.metadata.importance));
        
        let mut sorted_extra = BTreeMap::new();
        for (k, v) in &self.metadata.extra {
            sorted_extra.insert(k.clone(), v.clone());
        }
        deterministic_meta.insert("extra", serde_json::json!(sorted_extra));

        if let Ok(meta_json) = serde_json::to_string(&deterministic_meta) {
            hasher.update(meta_json.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }
}

impl TypedMetadata {
    pub fn to_string_with_template(&self, template: &str, separator: &str, excluded_keys: &[String]) -> String {
        let mut lines = Vec::new();
        
        let mut add_field = |key: &str, val: Option<String>| {
            if let Some(v) = val {
                if !excluded_keys.contains(&key.to_string()) {
                    lines.push(template.replace("{key}", key).replace("{value}", &v));
                }
            }
        };

        add_field("url", self.url.clone());
        add_field("file_path", self.file_path.clone());
        add_field("genre", self.genre.clone());
        if let Some(c) = self.confidence {
            add_field("confidence", Some(format!("{:.2}", c)));
        }
        if let Some(i) = self.importance {
            add_field("importance", Some(format!("{:.2}", i)));
        }
        if let Some(ts) = self.timestamp {
            add_field("timestamp", Some(ts.to_rfc3339()));
        }

        let mut sorted_extra: Vec<_> = self.extra.iter().collect();
        sorted_extra.sort_by_key(|k| k.0);

        for (key, value) in sorted_extra {
            if excluded_keys.contains(key) {
                continue;
            }
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            lines.push(template.replace("{key}", key).replace("{value}", &val_str));
        }

        lines.join(separator)
    }
}

impl Node {
    pub fn metadata_to_str(&self, context: MetadataContext) -> String {
        let excluded_keys = match context {
            MetadataContext::Embedding => &self.excluded_embed_metadata_keys,
            MetadataContext::Llm => &self.excluded_llm_metadata_keys,
            MetadataContext::General => &Vec::new(),
        };

        self.metadata.to_string_with_template(
            &self.metadata_template,
            &self.metadata_separator,
            excluded_keys
        )
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
