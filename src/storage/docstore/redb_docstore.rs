use crate::core::schema::Node;
use crate::storage::docstore::base::DocumentStore;
use anyhow::Result;
use async_trait::async_trait;
use redb::{Database, TableDefinition, ReadableTable};
use std::collections::HashMap;
use std::sync::Arc;

const DOCS_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("docs");
const HASH_TABLE: TableDefinition<&str, &str> = TableDefinition::new("hashes");

pub struct RedbDocumentStore {
    db: Arc<Database>,
}

impl RedbDocumentStore {
    pub fn new(path: &str) -> Result<Self> {
        let db = Database::create(path)?;

        // Initialize tables
        {
            let write_txn = db.begin_write()?;
            {
                let _ = write_txn.open_table(DOCS_TABLE)?;
                let _ = write_txn.open_table(HASH_TABLE)?;
            }
            write_txn.commit()?;
        }

        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl DocumentStore for RedbDocumentStore {
    async fn add_documents(&self, docs: Vec<Node>, _allow_update: bool) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DOCS_TABLE)?;
            for doc in docs {
                let bytes = bincode::serialize(&doc)?;
                table.insert(doc.id_.as_str(), bytes)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn get_document(&self, doc_id: &str) -> Result<Option<Node>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DOCS_TABLE)?;
        if let Some(bytes) = table.get(doc_id)? {
            let node: Node = bincode::deserialize(bytes.value().as_slice())?;
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    async fn delete_document(&self, doc_id: &str, _raise_error: bool) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DOCS_TABLE)?;
            table.remove(doc_id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn document_exists(&self, doc_id: &str) -> bool {
        let read_txn = self.db.begin_read().ok();
        if let Some(txn) = read_txn {
            if let Ok(table) = txn.open_table(DOCS_TABLE) {
                return table.get(doc_id).map(|res| res.is_some()).unwrap_or(false);
            }
        }
        false
    }

    async fn set_document_hash(&self, doc_id: &str, hash: &str) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(HASH_TABLE)?;
            table.insert(doc_id, hash)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn get_document_hash(&self, doc_id: &str) -> Result<Option<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(HASH_TABLE)?;
        Ok(table.get(doc_id)?.map(|h| h.value().to_string()))
    }

    async fn get_all_document_hashes(&self) -> Result<HashMap<String, String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(HASH_TABLE)?;
        let mut hashes = HashMap::new();
        for result in table.iter()? {
            let (id, hash) = result?;
            hashes.insert(id.value().to_string(), hash.value().to_string());
        }
        Ok(hashes)
    }

    async fn persist(&self, _path: &str) -> Result<()> {
        Ok(()) // Redb is persistent
    }
}
