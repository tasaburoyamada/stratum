use serde::{Deserialize, Serialize};
use half::f16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryBundle {
    pub query_str: String,
    pub embedding: Option<Vec<f16>>,
    pub custom_embedding_strs: Option<Vec<String>>,
}

impl QueryBundle {
    pub fn new(query_str: String) -> Self {
        Self {
            query_str,
            embedding: None,
            custom_embedding_strs: None,
        }
    }
}
