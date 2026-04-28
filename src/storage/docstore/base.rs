use async_trait::async_trait;
use crate::core::schema::Node;
use anyhow::Result;
use std::collections::HashMap;

#[async_trait]
pub trait DocumentStore: Send + Sync {
    async fn add_documents(&self, docs: Vec<Node>, allow_update: bool) -> Result<()>;
    async fn get_document(&self, doc_id: &str) -> Result<Option<Node>>;
    async fn delete_document(&self, doc_id: &str, raise_error: bool) -> Result<()>;
    async fn document_exists(&self, doc_id: &str) -> bool;
    
    // Hash management for deduplication
    async fn set_document_hash(&self, doc_id: &str, hash: &str) -> Result<()>;
    async fn get_document_hash(&self, doc_id: &str) -> Result<Option<String>>;
    async fn get_all_document_hashes(&self) -> Result<HashMap<String, String>>;
    
    async fn persist(&self, path: &str) -> Result<()>;
}
