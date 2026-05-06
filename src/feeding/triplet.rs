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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorTriplet {
    pub query: String,
    pub parent_context: String,
    pub choices: Vec<String>,
    pub selected_indices: Vec<usize>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl SelectorTriplet {
    pub fn new(query: String, parent_context: String, choices: Vec<String>, selected_indices: Vec<usize>) -> Self {
        Self {
            query,
            parent_context,
            choices,
            selected_indices,
            metadata: std::collections::HashMap::new(),
        }
    }
}
