use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexStruct {
    pub index_id: String,
    pub summary: Option<String>,
    pub nodes_dict: HashMap<String, String>, // node_id -> doc_id
}

#[async_trait]
pub trait IndexStore: Send + Sync {
    async fn add_index_struct(&self, index_struct: IndexStruct) -> Result<()>;
    async fn get_index_struct(&self, index_id: &str) -> Result<Option<IndexStruct>>;
    async fn persist(&self, path: &str) -> Result<()>;
}

pub struct SimpleIndexStore {
    data: RwLock<HashMap<String, IndexStruct>>,
}

impl SimpleIndexStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl IndexStore for SimpleIndexStore {
    async fn add_index_struct(&self, index_struct: IndexStruct) -> Result<()> {
        let mut data = self.data.write().map_err(|_| anyhow::anyhow!("ERR_LOCK_POISONED"))?;
        data.insert(index_struct.index_id.clone(), index_struct);
        Ok(())
    }

    async fn get_index_struct(&self, index_id: &str) -> Result<Option<IndexStruct>> {
        let data = self.data.read().map_err(|_| anyhow::anyhow!("ERR_LOCK_POISONED"))?;
        Ok(data.get(index_id).cloned())
    }

    async fn persist(&self, path: &str) -> Result<()> {
        let data = self.data.read().map_err(|_| anyhow::anyhow!("ERR_LOCK_POISONED"))?;
        let encoded: Vec<u8> = bincode::serialize(&*data)?;
        
        let tmp_path = format!("{}.tmp", path);
        std::fs::write(&tmp_path, encoded)?;
        std::fs::rename(tmp_path, path)?;
        
        Ok(())
    }
}
