#![cfg(feature = "persistence")]
use crate::core::schema::Node;
use crate::storage::docstore::base::DocumentStore;
use crate::storage::docstore::types::RefDocInfo;
use anyhow::Result;
use async_trait::async_trait;
use redb::{Database, TableDefinition, ReadableTable};
use std::collections::HashMap;
use std::sync::Arc;
use log::info;

const DOCS_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("docs");
const HASH_TABLE: TableDefinition<&str, &str> = TableDefinition::new("hashes");
const REF_DOC_INFO_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("ref_doc_info");

pub struct RedbDocumentStore {
    db: Arc<Database>,
}

impl RedbDocumentStore {
    pub fn new(path: &str) -> Result<Self> {
        info!("RedbDocumentStore::new: path={}", path);
        let db = if std::path::Path::new(path).exists() {
            info!("RedbDocumentStore::new: Opening existing database at {}", path);
            Database::open(path)?
        } else {
            info!("RedbDocumentStore::new: Creating new database at {}", path);
            Database::create(path)?
        };

        {
            let write_txn = db.begin_write()?;
            {
                let _ = write_txn.open_table(DOCS_TABLE)?;
                let _ = write_txn.open_table(HASH_TABLE)?;
                let _ = write_txn.open_table(REF_DOC_INFO_TABLE)?;
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
                info!("RedbDocumentStore::add_documents: Adding document with ID {}", doc.id_);
                let bytes = serde_json::to_vec(&doc)?;
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
            info!("RedbDocumentStore::get_document: Found document with ID {}", doc_id);
            let node: Node = serde_json::from_slice(bytes.value().as_slice())?;
            Ok(Some(node))
        } else {
            info!("RedbDocumentStore::get_document: Document with ID {} not found.", doc_id);
            Ok(None)
        }
    }

    async fn delete_document(&self, doc_id: &str, _raise_error: bool) -> Result<()> {
        info!("RedbDocumentStore::delete_document: Deleting document with ID {}", doc_id);
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
                let exists = table.get(doc_id).map(|res| res.is_some()).unwrap_or(false);
                info!("RedbDocumentStore::document_exists: Document with ID {} exists: {}", doc_id, exists);
                return exists;
            }
        }
        info!("RedbDocumentStore::document_exists: Failed to check existence for ID {}. Assuming false.", doc_id);
        false
    }

    async fn set_document_hash(&self, doc_id: &str, hash: &str) -> Result<()> {
        info!("RedbDocumentStore::set_document_hash: Setting hash for ID {}", doc_id);
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
        let hash = table.get(doc_id)?.map(|h| h.value().to_string());
        info!("RedbDocumentStore::get_document_hash: Retrieved hash for ID {}: {:?}", doc_id, hash);
        Ok(hash)
    }

    async fn get_all_document_hashes(&self) -> Result<HashMap<String, String>> {
        info!("RedbDocumentStore::get_all_document_hashes: Retrieving all hashes.");
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(HASH_TABLE)?;
        let mut hashes = HashMap::new();
        for result in table.iter()? {
            let (id, hash) = result?;
            hashes.insert(id.value().to_string(), hash.value().to_string());
        }
        info!("RedbDocumentStore::get_all_document_hashes: Found {} hashes.", hashes.len());
        Ok(hashes)
    }

    async fn get_ref_doc_info(&self, ref_doc_id: &str) -> Result<Option<RefDocInfo>> {
        info!("RedbDocumentStore::get_ref_doc_info: Getting ref doc info for ID {}", ref_doc_id);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(REF_DOC_INFO_TABLE)?;
        if let Some(bytes) = table.get(ref_doc_id)? {
            let info = serde_json::from_slice(bytes.value().as_slice())?;
            info!("RedbDocumentStore::get_ref_doc_info: Found ref doc info for ID {}", ref_doc_id);
            Ok(Some(info))
        } else {
            info!("RedbDocumentStore::get_ref_doc_info: Ref doc info for ID {} not found.", ref_doc_id);
            Ok(None)
        }
    }

    async fn set_ref_doc_info(&self, ref_doc_id: &str, ref_doc_info: RefDocInfo) -> Result<()> {
        info!("RedbDocumentStore::set_ref_doc_info: Setting ref doc info for ID {}", ref_doc_id);
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(REF_DOC_INFO_TABLE)?;
            let bytes = serde_json::to_vec(&ref_doc_info)?;
            table.insert(ref_doc_id, bytes)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn delete_ref_doc_info(&self, ref_doc_id: &str) -> Result<()> {
        info!("RedbDocumentStore::delete_ref_doc_info: Deleting ref doc info for ID {}", ref_doc_id);
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(REF_DOC_INFO_TABLE)?;
            table.remove(ref_doc_id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn get_all_ref_doc_info(&self) -> Result<HashMap<String, RefDocInfo>> {
        info!("RedbDocumentStore::get_all_ref_doc_info: Retrieving all ref doc infos.");
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(REF_DOC_INFO_TABLE)?;
        let mut infos = HashMap::new();
        for result in table.iter()? {
            let (id, bytes) = result?;
            let info = serde_json::from_slice(bytes.value().as_slice())?;
            infos.insert(id.value().to_string(), info);
        }
        info!("RedbDocumentStore::get_all_ref_doc_info: Found {} ref doc infos.", infos.len());
        Ok(infos)
    }

    async fn persist(&self, _path: &str) -> Result<()> {
        info!("RedbDocumentStore::persist: No-op for RedbDocumentStore as it's persistent.");
        Ok(()) // Redb is persistent
    }
}
