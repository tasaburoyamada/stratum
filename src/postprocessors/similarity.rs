use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::postprocessors::base::NodePostprocessor;
use anyhow::Result;
use async_trait::async_trait;

pub struct SimilarityPostprocessor {
    pub similarity_threshold: f32,
}

impl SimilarityPostprocessor {
    pub fn new(similarity_threshold: f32) -> Self {
        Self { similarity_threshold }
    }
}

#[async_trait]
impl NodePostprocessor for SimilarityPostprocessor {
    async fn postprocess_nodes(&self, nodes: Vec<NodeWithScore>, _query_bundle: &QueryBundle) -> Result<Vec<NodeWithScore>> {
        let filtered_nodes = nodes.into_iter()
            .filter(|node| {
                if let Some(score) = node.score {
                    score >= self.similarity_threshold
                } else {
                    false
                }
            })
            .collect();
        Ok(filtered_nodes)
    }
}
