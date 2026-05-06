use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use crate::llm::LlmClient;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// A parser that uses LLM to transform unstructured text (e.g. NGA/OCR output)
/// into a structured JSON format based on a provided schema or template.
#[derive(Clone)]
pub struct RefineryParser {
    llm: Arc<dyn LlmClient>,
    schema_description: String,
}

impl std::fmt::Debug for RefineryParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefineryParser")
            .field("schema_description", &self.schema_description)
            .finish()
    }
}

impl RefineryParser {
    pub fn new(llm: Arc<dyn LlmClient>, schema_description: String) -> Self {
        Self { llm, schema_description }
    }

    pub fn salesforce_exam_default(llm: Arc<dyn LlmClient>) -> Self {
        let schema = "
        {
          \"id\": \"String (Unique ID)\",
          \"exam_id\": \"String\",
          \"question\": \"String (The exam question)\",
          \"options\": { \"A\": \"...\", \"B\": \"...\" },
          \"answer\": [\"A\"],
          \"explanation\": \"String\",
          \"metadata\": { \"genre\": \"...\", \"version\": \"...\" }
        }";
        Self::new(llm, schema.to_string())
    }
}

#[async_trait]
impl Transformation for RefineryParser {
    async fn transform(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        let mut refined_nodes = Vec::new();

        for node in nodes {
            if let crate::core::schema::NodeContent::Text(raw_text) = &node.content {
                let prompt = format!(
                    "Extract structured information from the following raw text based on the provided JSON schema.\n\
                    Schema:\n{}\n\n\
                    Raw Text:\n{}\n\n\
                    Respond with ONLY the valid JSON object.",
                    self.schema_description,
                    raw_text
                );

                if let Ok(refined_json) = self.llm.complete(&prompt).await {
                    // Create a new node with structured JSON as content (stored as Text for now, or we could add JSON variant)
                    let mut refined_node = Node::new_text(refined_json.trim().to_string());
                    refined_node.metadata = node.metadata.clone();
                    refined_node.metadata.extra.insert("is_refined".to_string(), serde_json::json!(true));
                    refined_nodes.push(refined_node);
                } else {
                    // If refinement fails, keep the original or log error
                    refined_nodes.push(node);
                }
            } else {
                refined_nodes.push(node);
            }
        }

        Ok(refined_nodes)
    }

    fn hash(&self) -> String {
        format!("refinery_parser_{}", blake3::hash(self.schema_description.as_bytes()))
    }
}
