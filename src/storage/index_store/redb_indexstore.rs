#![cfg(feature = "persistence")]
use crate::storage::index_store::base::{IndexStore, IndexStruct};
use anyhow::Result;
use async_trait::async_trait;
use redb::{Database, TableDefinition};
use std::sync::Arc;

const INDEX_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("indexes");

pub struct RedbIndexStore {
    db: Arc<Database>,
}

impl RedbIndexStore {
    pub fn new(path: &str) -> Result<Self> {
        let db = Database::create(path)?;
        {
            let write_txn = db.begin_write()?;
            {
                let _ = write_txn.open_table(INDEX_TABLE)?;
            }
            write_txn.commit()?;
        }
        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl IndexStore for RedbIndexStore {
    async fn add_index_struct(&self, index_struct: IndexStruct) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(INDEX_TABLE)?;
            let bytes = serde_json::to_vec(&index_struct)?;
            table.insert(index_struct.index_id.as_str(), bytes)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn get_index_struct(&self, index_id: &str) -> Result<Option<IndexStruct>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(INDEX_TABLE)?;
        if let Some(bytes) = table.get(index_id)? {
            let index_struct: IndexStruct = serde_json::from_slice(bytes.value().as_slice())?;
            Ok(Some(index_struct))
        } else {
            Ok(None)
        }
    }

    async fn persist(&self, _path: &str) -> Result<()> {
        Ok(()) // Redb is persistent
    }
}
