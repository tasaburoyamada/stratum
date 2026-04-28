use async_trait::async_trait;
use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use anyhow::Result;

#[async_trait]
pub trait NodePostprocessor: Send + Sync {
    async fn postprocess_nodes(&self, nodes: Vec<NodeWithScore>, query_bundle: &QueryBundle) -> Result<Vec<NodeWithScore>>;
}
