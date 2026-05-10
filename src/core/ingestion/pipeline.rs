use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use anyhow::Result;
use std::sync::Arc;
use indicatif::ProgressBar;

pub struct IngestionPipeline {
    pub transformations: Vec<Arc<dyn Transformation>>,
    pub docstore: Option<Arc<dyn crate::storage::docstore::base::DocumentStore>>,
    pub cache: Option<std::sync::Arc<std::sync::Mutex<super::cache::IngestionCache>>>,
}

impl IngestionPipeline {
    pub fn new(
        transformations: Vec<Arc<dyn Transformation>>, 
        docstore: Option<Arc<dyn crate::storage::docstore::base::DocumentStore>>
    ) -> Self {
        Self { transformations, docstore, cache: None }
    }

    pub fn with_cache(mut self, cache: std::sync::Arc<std::sync::Mutex<super::cache::IngestionCache>>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub async fn run(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        let mut nodes_to_process = Vec::new();
        let mut processed_info = Vec::new();

        // 1. Deduplication (Phase 0)
        for node in nodes {
            let hash = node.hash();
            let mut exists = false;

            // Use Pipeline Cache first for speed
            if let Some(cache_lock) = &self.cache {
                let mut cache = cache_lock.lock().map_err(|_| anyhow::anyhow!("ERR_LOCK_POISONED"))?;
                if cache.check_and_update(&node.id_, &hash) {
                    log::info!("Node {} unchanged in cache. Skipping.", node.id_);
                    exists = true;
                }
            }

            // Fallback to DocStore if no cache or double verification
            if !exists {
                if let Some(ds) = &self.docstore {
                    if let Ok(Some(existing_hash)) = ds.get_document_hash(&node.id_).await {
                        if existing_hash == hash {
                            log::info!("Node {} already in DocStore. Skipping.", node.id_);
                            exists = true;
                        }
                    }
                }
            }

            if !exists {
                processed_info.push((node.id_.clone(), hash));
                nodes_to_process.push(node);
            }
        }

        if nodes_to_process.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Apply transformations
        let mut current_nodes = nodes_to_process;
        let pb = ProgressBar::new(self.transformations.len() as u64);

        for transform in &self.transformations {
            current_nodes = transform.transform(current_nodes).await?;
            pb.inc(1);
        }

        // 3. Update hashes for ORIGINAL nodes in docstore (Phase 2)
        if let Some(ds) = &self.docstore {
            for (id, hash) in processed_info {
                ds.set_document_hash(&id, &hash).await?;
            }
        }

        pb.finish_with_message("Ingestion completed.");
        Ok(current_nodes)
    }
}
