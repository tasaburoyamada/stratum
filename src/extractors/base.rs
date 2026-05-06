use async_trait::async_trait;
use crate::core::schema::Node;
use anyhow::Result;

#[async_trait]
pub trait MetadataExtractor: Send + Sync {
    /// Extract metadata from a node and update it.
    async fn extract(&self, nodes: Vec<Node>) -> Result<Vec<Node>>;
}
