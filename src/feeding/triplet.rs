use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetTriplet {
    pub instruction: String,
    pub input: String,
    pub output: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl DatasetTriplet {
    pub fn new(query: String, context: String, response: String) -> Self {
        Self {
            instruction: "You are an assistant. Answer the question based on the provided context.".to_string(),
            input: format!("Context: {}\n\nQuestion: {}", context, query),
            output: response,
            metadata: std::collections::HashMap::new(),
        }
    }
}
