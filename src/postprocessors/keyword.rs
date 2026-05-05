use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::postprocessors::base::NodePostprocessor;
use anyhow::Result;
use async_trait::async_trait;

pub struct KeywordPostprocessor {
    pub required_keywords: Vec<String>,
    pub excluded_keywords: Vec<String>,
}

impl KeywordPostprocessor {
    pub fn new(required_keywords: Vec<String>, excluded_keywords: Vec<String>) -> Self {
        Self { required_keywords, excluded_keywords }
    }
}

#[async_trait]
impl NodePostprocessor for KeywordPostprocessor {
    async fn postprocess_nodes(&self, nodes: Vec<NodeWithScore>, _query_bundle: &QueryBundle) -> Result<Vec<NodeWithScore>> {
        let filtered_nodes = nodes.into_iter()
            .filter(|node| {
                if let Ok(content) = node.node.get_content(None) {
                    let content_lower = content.to_lowercase();
                    
                    // Check required
                    for kw in &self.required_keywords {
                        if !content_lower.contains(&kw.to_lowercase()) {
                            return false;
                        }
                    }
                    
                    // Check excluded
                    for kw in &self.excluded_keywords {
                        if content_lower.contains(&kw.to_lowercase()) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            })
            .collect();
        Ok(filtered_nodes)
    }
}
