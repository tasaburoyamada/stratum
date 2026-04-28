use async_trait::async_trait;
use crate::core::schema::Node;
use anyhow::Result;
use std::fmt::Debug;

#[async_trait]
pub trait Transformation: Send + Sync + Debug {
    async fn transform(&self, nodes: Vec<Node>) -> Result<Vec<Node>>;
    
    /// Returns a unique hash for this transformation and its configuration
    fn hash(&self) -> String;
}
