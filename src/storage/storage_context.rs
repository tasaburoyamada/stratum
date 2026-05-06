use crate::storage::docstore::base::DocumentStore;
use crate::storage::index_store::IndexStore;
use crate::vector_stores::base::VectorStore;
use crate::storage::docstore::simple::SimpleDocumentStore;
use crate::storage::docstore::redb_docstore::RedbDocumentStore;
use crate::storage::index_store::{SimpleIndexStore, RedbIndexStore};
use crate::vector_stores::simple::SimpleVectorStore;
use crate::vector_stores::native::NativeVectorStore;
use anyhow::Result;
use std::sync::Arc;
use std::path::Path;

#[derive(Clone)]
pub struct StorageContext {
    pub docstore: Arc<dyn DocumentStore>,
    pub vector_store: Arc<dyn VectorStore>,
    pub index_store: Arc<dyn IndexStore>,
}

impl StorageContext {
    pub fn new(
        docstore: Arc<dyn DocumentStore>, 
        vector_store: Arc<dyn VectorStore>,
        index_store: Arc<dyn IndexStore>,
    ) -> Self {
        Self {
            docstore,
            vector_store,
            index_store,
        }
    }

    pub fn from_defaults() -> Self {
        Self {
            docstore: Arc::new(SimpleDocumentStore::new()),
            vector_store: Arc::new(SimpleVectorStore::new()),
            index_store: Arc::new(SimpleIndexStore::new()),
        }
    }

    pub fn from_dir(persist_dir: &str) -> Result<Self> {
        let docstore_path = format!("{}/docstore.bin", persist_dir);
        let redb_docstore_path = format!("{}/docstore.redb", persist_dir);
        let index_store_path = format!("{}/index_store.bin", persist_dir);
        let redb_index_store_path = format!("{}/index_store.redb", persist_dir);
        let vector_store_path = format!("{}/vector_store.bin", persist_dir);
        let native_db_path = format!("{}/native_db.redb", persist_dir);

        let docstore = if Path::new(&redb_docstore_path).exists() {
            Arc::new(RedbDocumentStore::new(&redb_docstore_path)?) as Arc<dyn DocumentStore>
        } else if Path::new(&docstore_path).exists() {
            Arc::new(SimpleDocumentStore::load(&docstore_path)?) as Arc<dyn DocumentStore>
        } else {
            Arc::new(RedbDocumentStore::new(&redb_docstore_path)?) as Arc<dyn DocumentStore>
        };

        let index_store = if Path::new(&redb_index_store_path).exists() {
            Arc::new(RedbIndexStore::new(&redb_index_store_path)?) as Arc<dyn IndexStore>
        } else if Path::new(&index_store_path).exists() {
            Arc::new(SimpleIndexStore::load(&index_store_path)?) as Arc<dyn IndexStore>
        } else {
            Arc::new(RedbIndexStore::new(&redb_index_store_path)?) as Arc<dyn IndexStore>
        };

        let vector_store = if Path::new(&native_db_path).exists() {
            Arc::new(NativeVectorStore::new(&native_db_path, 384)?) as Arc<dyn VectorStore>
        } else if Path::new(&vector_store_path).exists() {
            Arc::new(SimpleVectorStore::load(&vector_store_path)?) as Arc<dyn VectorStore>
        } else {
            Arc::new(NativeVectorStore::new(&native_db_path, 384)?) as Arc<dyn VectorStore>
        };

        Ok(Self {
            docstore,
            vector_store,
            index_store,
        })
    }

    pub async fn persist(&self, persist_dir: &str) -> Result<()> {
        std::fs::create_dir_all(persist_dir)?;
        
        self.docstore.persist(&format!("{}/docstore.redb", persist_dir)).await?;
        self.index_store.persist(&format!("{}/index_store.redb", persist_dir)).await?;
        self.vector_store.persist(&format!("{}/native_db.redb", persist_dir)).await?;
        
        Ok(())
    }

    pub async fn export_dataset(&self, output_path: &str) -> Result<()> {
        let mut file = std::fs::File::create(output_path)?;
        let hashes = self.docstore.get_all_document_hashes().await?;
        
        for doc_id in hashes.keys() {
            if let Some(node) = self.docstore.get_document(doc_id).await? {
                let json = serde_json::to_string(&node)?;
                use std::io::Write;
                writeln!(file, "{}", json)?;
            }
        }
        
        log::info!("Exported dataset to: {}", output_path);
        Ok(())
    }
}
