use crate::storage::docstore::base::DocumentStore;
use crate::storage::index_store::IndexStore;
use crate::vector_stores::base::VectorStore;
use crate::storage::docstore::simple::SimpleDocumentStore;
#[cfg(feature = "persistence")]
use crate::storage::docstore::redb_docstore::RedbDocumentStore;
use crate::storage::index_store::SimpleIndexStore;
#[cfg(feature = "persistence")]
use crate::storage::index_store::RedbIndexStore;
use crate::vector_stores::simple::SimpleVectorStore;
#[cfg(feature = "persistence")]
use crate::vector_stores::native::NativeVectorStore;
use anyhow::Result;
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Debug, Default)]
struct FoundStores {
    docstore: bool,
    index_store: bool,
    vector_store: bool,
}

impl FoundStores {
    fn is_complete(&self) -> bool {
        self.docstore && self.index_store && self.vector_store
    }

    fn missing_report(&self) -> String {
        let mut missing = Vec::new();
        if !self.docstore { missing.push("docstore"); }
        if !self.index_store { missing.push("index_store"); }
        if !self.vector_store { missing.push("vector_store"); }
        missing.join(", ")
    }
}

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

    /// Initializes high-speed in-memory storage.
    pub fn in_memory() -> Self {
        Self {
            docstore: Arc::new(SimpleDocumentStore::new()),
            vector_store: Arc::new(SimpleVectorStore::new()),
            index_store: Arc::new(SimpleIndexStore::new()),
        }
    }

    pub fn from_defaults() -> Self {
        Self::in_memory()
    }
}

impl StorageContext {
    pub fn from_dir(persist_dir: &str) -> Result<Self> {
        let root = PathBuf::from(persist_dir);
        let mut found = FoundStores::default();

        let bin_docstore_path = root.join("docstore.bin");
        let bin_index_store_path = root.join("index_store.bin");
        let bin_vector_store_path = root.join("vector_store.bin");

        let redb_docstore_path = root.join("docstore.redb");
        let redb_index_store_path = root.join("index_store.redb");
        let native_db_path = root.join("native_db.redb");

        // 1. Resolve Document Store
        let doc_exists_redb = redb_docstore_path.exists();
        let doc_exists_bin = bin_docstore_path.exists();
        if doc_exists_redb || doc_exists_bin { found.docstore = true; }

        let docstore: Arc<dyn DocumentStore> = if doc_exists_redb {
            #[cfg(feature = "persistence")]
            { Arc::new(RedbDocumentStore::new(&redb_docstore_path.to_string_lossy())?) }
            #[cfg(not(feature = "persistence"))]
            { return Err(anyhow::anyhow!("Detected redb document store but 'persistence' feature is disabled")); }
        } else if doc_exists_bin {
            Arc::new(SimpleDocumentStore::load(&bin_docstore_path.to_string_lossy())?)
        } else {
            #[cfg(feature = "persistence")]
            { Arc::new(RedbDocumentStore::new(&redb_docstore_path.to_string_lossy())?) }
            #[cfg(not(feature = "persistence"))]
            { Arc::new(SimpleDocumentStore::new()) }
        };

        // 2. Resolve Index Store
        let idx_exists_redb = redb_index_store_path.exists();
        let idx_exists_bin = bin_index_store_path.exists();
        if idx_exists_redb || idx_exists_bin { found.index_store = true; }

        let index_store: Arc<dyn IndexStore> = if idx_exists_redb {
            #[cfg(feature = "persistence")]
            { Arc::new(RedbIndexStore::new(&redb_index_store_path.to_string_lossy())?) }
            #[cfg(not(feature = "persistence"))]
            { return Err(anyhow::anyhow!("Detected redb index store but 'persistence' feature is disabled")); }
        } else if idx_exists_bin {
            Arc::new(SimpleIndexStore::load(&bin_index_store_path.to_string_lossy())?)
        } else {
            #[cfg(feature = "persistence")]
            { Arc::new(RedbIndexStore::new(&redb_index_store_path.to_string_lossy())?) }
            #[cfg(not(feature = "persistence"))]
            { Arc::new(SimpleIndexStore::new()) }
        };

        // 3. Resolve Vector Store
        let vec_exists_redb = native_db_path.exists();
        let vec_exists_bin = bin_vector_store_path.exists();
        if vec_exists_redb || vec_exists_bin { found.vector_store = true; }

        let vector_store: Arc<dyn VectorStore> = if vec_exists_redb {
            #[cfg(feature = "persistence")]
            { Arc::new(NativeVectorStore::new(&native_db_path.to_string_lossy(), 0)?) }
            #[cfg(not(feature = "persistence"))]
            { return Err(anyhow::anyhow!("Detected native vector store (redb) but 'persistence' feature is disabled")); }
        } else if vec_exists_bin {
            Arc::new(SimpleVectorStore::load(&bin_vector_store_path.to_string_lossy())?)
        } else {
            #[cfg(feature = "persistence")]
            { Arc::new(NativeVectorStore::new(&native_db_path.to_string_lossy(), 384)?) }
            #[cfg(not(feature = "persistence"))]
            { Arc::new(SimpleVectorStore::new()) }
        };

        // 4. Validate Store Consistency
        if !found.is_complete() && (found.docstore || found.index_store || found.vector_store) {
            log::warn!("Incomplete storage found in {}. Missing: {}. This might lead to inconsistent data.", persist_dir, found.missing_report());
        }

        Ok(Self { docstore, vector_store, index_store })
    }

    pub async fn persist(&self, persist_dir: &str) -> Result<()> {
        std::fs::create_dir_all(persist_dir)?;
        
        #[cfg(feature = "persistence")]
        {
            self.docstore.persist(&format!("{}/docstore.redb", persist_dir)).await?;
            self.index_store.persist(&format!("{}/index_store.redb", persist_dir)).await?;
            self.vector_store.persist(&format!("{}/native_db.redb", persist_dir)).await?;
        }

        #[cfg(not(feature = "persistence"))]
        {
            self.docstore.persist(&format!("{}/docstore.bin", persist_dir)).await?;
            self.index_store.persist(&format!("{}/index_store.bin", persist_dir)).await?;
            self.vector_store.persist(&format!("{}/vector_store.bin", persist_dir)).await?;
        }
        
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
