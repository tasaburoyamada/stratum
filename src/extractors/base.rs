use async_trait::async_trait;
use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use anyhow::Result;
use std::fmt::Debug;

#[async_trait]
pub trait MetadataExtractor: Transformation + Send + Sync + Debug {
    /// Extract metadata from a node and update it.
    async fn extract(&self, nodes: Vec<Node>) -> Result<Vec<Node>>;
}

