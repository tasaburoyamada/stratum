use crate::core::schema::Node;
use crate::storage::docstore::base::DocumentStore;
use crate::storage::docstore::types::RefDocInfo;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SimpleDocStoreData {
    docs: HashMap<String, Node>,
    hashes: HashMap<String, String>,
    ref_doc_info: HashMap<String, RefDocInfo>,
}

pub struct SimpleDocumentStore {
    data: RwLock<SimpleDocStoreData>,
}

impl SimpleDocumentStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(SimpleDocStoreData {
                docs: HashMap::new(),
                hashes: HashMap::new(),
                ref_doc_info: HashMap::new(),
            }),
        }
    }

    /// FOR TESTING ONLY: Trigger a panic while holding the write lock to test poisoning.
    pub fn force_poison(&self) {
        let _lock = self.data.write().unwrap();
        panic!("Intentional panic to poison lock");
    }
}

#[async_trait]
impl DocumentStore for SimpleDocumentStore {
    async fn add_documents(&self, docs: Vec<Node>, allow_update: bool) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        for doc in docs {
            if !allow_update && data.docs.contains_key(&doc.id_) {
                continue;
            }
            // Update hash automatically from Node
            data.hashes.insert(doc.id_.clone(), doc.hash());
            data.docs.insert(doc.id_.clone(), doc);
        }
        Ok(())
    }

    async fn get_document(&self, doc_id: &str) -> Result<Option<Node>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        Ok(data.docs.get(doc_id).cloned())
    }

    async fn delete_document(&self, doc_id: &str, _raise_error: bool) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        data.docs.remove(doc_id);
        data.hashes.remove(doc_id);
        Ok(())
    }

    async fn document_exists(&self, doc_id: &str) -> bool {
        let data = self.data.read();
        if let Ok(d) = data {
            d.docs.contains_key(doc_id)
        } else {
            false
        }
    }

    async fn set_document_hash(&self, doc_id: &str, hash: &str) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        data.hashes.insert(doc_id.to_string(), hash.to_string());
        Ok(())
    }

    async fn get_document_hash(&self, doc_id: &str) -> Result<Option<String>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        Ok(data.hashes.get(doc_id).cloned())
    }

    async fn get_all_document_hashes(&self) -> Result<HashMap<String, String>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        Ok(data.hashes.clone())
    }

    async fn persist(&self, path: &str) -> Result<()> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("ERR_LOCK_POISONED: {}", e))?;
        let encoded: Vec<u8> = bincode::serialize(&*data)?;
        
        let tmp_path = format!("{}.tmp", path);
        std::fs::write(&tmp_path, encoded)?;
        std::fs::rename(tmp_path, path)?;
        
        Ok(())
    }
}
