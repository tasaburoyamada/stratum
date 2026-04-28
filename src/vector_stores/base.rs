use async_trait::async_trait;
use crate::core::schema::Node;
use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryResult};
use anyhow::Result;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn add(&self, nodes: Vec<Node>) -> Result<Vec<String>>;
    async fn delete(&self, ref_doc_id: &str) -> Result<()>;
    async fn query(&self, query: VectorStoreQuery) -> Result<VectorStoreQueryResult>;
    async fn persist(&self, path: &str) -> Result<()>;
}
