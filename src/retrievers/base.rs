use async_trait::async_trait;
use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use anyhow::Result;

#[async_trait]
pub trait Retriever: Send + Sync {
    async fn retrieve(&self, query_bundle: QueryBundle) -> Result<Vec<NodeWithScore>>;
}
