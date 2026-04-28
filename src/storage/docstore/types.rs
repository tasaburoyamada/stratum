use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefDocInfo {
    pub node_ids: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
