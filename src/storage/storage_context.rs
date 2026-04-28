use crate::storage::docstore::base::DocumentStore;
use crate::storage::index_store::IndexStore;
use crate::vector_stores::base::VectorStore;
use anyhow::Result;
use std::sync::Arc;

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

    pub async fn persist(&self, persist_dir: &str) -> Result<()> {
        std::fs::create_dir_all(persist_dir)?;
        
        self.docstore.persist(&format!("{}/docstore.json", persist_dir)).await?;
        self.index_store.persist(&format!("{}/index_store.json", persist_dir)).await?;
        self.vector_store.persist(&format!("{}/vector_store.json", persist_dir)).await?;
        
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
