#![cfg(feature = "persistence")]
use crate::core::schema::Node;
use crate::vector_stores::base::VectorStore;
use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryResult};
use anyhow::Result;
use async_trait::async_trait;
use hnsw_rs::prelude::*;
use redb::{Database, TableDefinition, ReadableTable};
use std::sync::{Arc, RwLock};
use rayon::prelude::*;

// Optimization: Store embeddings separately for faster index rebuild
const EMBEDDINGS_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("embeddings");
const MAPPING_TABLE: TableDefinition<u64, &str> = TableDefinition::new("mapping");
const REVERSE_MAPPING_TABLE: TableDefinition<&str, u64> = TableDefinition::new("reverse_mapping");
const NODES_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("nodes");
const METADATA_TABLE: TableDefinition<&str, u64> = TableDefinition::new("metadata");

// Constants for tuning HNSW behavior
const DEFAULT_DIM: usize = 384;
const FILTER_SEARCH_MULTIPLIER: usize = 10;
const MIN_SEARCH_K: usize = 100;
const MAX_SEARCH_K: usize = 5000;
const HNSW_EF_CONSTRUCTION: usize = 200;
const HNSW_M: usize = 16;

pub struct NativeVectorStore {
    db: Database,
    hnsw: Arc<RwLock<Hnsw<'static, f32, DistCosine>>>,
    dim: usize,
}

impl NativeVectorStore {
    pub fn new(path: &str, dim: usize) -> Result<Self> {
        let db = if std::path::Path::new(path).exists() {
            Database::open(path)?
        } else {
            Database::create(path)?
        };

        // Initialize tables and check dimension
        let final_dim = {
            let write_txn = db.begin_write()?;
            let resolved_dim = {
                let _ = write_txn.open_table(EMBEDDINGS_TABLE)?;
                let _ = write_txn.open_table(MAPPING_TABLE)?;
                let _ = write_txn.open_table(REVERSE_MAPPING_TABLE)?;
                let _ = write_txn.open_table(NODES_TABLE)?;
                let mut meta_table = write_txn.open_table(METADATA_TABLE)?;
                
                let stored_dim = meta_table.get("dim")?.map(|g| g.value() as usize);
                
                if let Some(s_dim) = stored_dim {
                    if s_dim != dim && dim != 0 {
                        return Err(anyhow::anyhow!("Dimension mismatch: stored={}, requested={}", s_dim, dim));
                    }
                    s_dim
                } else {
                    let d = if dim == 0 { DEFAULT_DIM } else { dim };
                    meta_table.insert("dim", d as u64)?;
                    d
                }
            };
            write_txn.commit()?;
            resolved_dim
        };

        let hnsw = Hnsw::new(HNSW_M, 100, HNSW_M, HNSW_EF_CONSTRUCTION, DistCosine);
        let hnsw = Arc::new(RwLock::new(hnsw));

        let store = Self { db, hnsw, dim: final_dim };
        store.rebuild_index()?;

        Ok(store)
    }

    fn rebuild_index(&self) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let embeddings_table = read_txn.open_table(EMBEDDINGS_TABLE)?;
        let reverse_table = read_txn.open_table(REVERSE_MAPPING_TABLE)?;
        
        let mut raw_entries = Vec::new();

        for result in embeddings_table.iter()? {
            let (id, emb_bytes) = result?;
            let embedding: Vec<half::f16> = serde_json::from_slice(emb_bytes.value().as_slice())?;
            
            if embedding.len() != self.dim {
                log::error!("Embedding dimension mismatch for node {}: expected {}, found {}", id.value(), self.dim, embedding.len());
                continue;
            }

            if let Some(inner_id) = reverse_table.get(id.value())? {
                raw_entries.push((embedding, inner_id.value() as usize));
            }
        }
        
        // Parallel conversion using Rayon
        let entries: Vec<(Vec<f32>, usize)> = raw_entries.into_par_iter()
            .map(|(emb_f16, id)| {
                let emb_f32: Vec<f32> = emb_f16.into_iter().map(f32::from).collect();
                (emb_f32, id)
            })
            .collect();
        
        // Tight lock window for HNSW insertion
        {
            let hnsw = self.hnsw.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            for (emb, id) in entries {
                hnsw.insert((&emb, id));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl VectorStore for NativeVectorStore {
    async fn add(&self, nodes: Vec<Node>) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        let mut pending_index_updates = Vec::new();

        // 1. Acquire HNSW lock FIRST to prevent race with query
        let hnsw = self.hnsw.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        let write_txn = self.db.begin_write()?;
        {
            let mut emb_table = write_txn.open_table(EMBEDDINGS_TABLE)?;
            let mut mapping_table = write_txn.open_table(MAPPING_TABLE)?;
            let mut reverse_table = write_txn.open_table(REVERSE_MAPPING_TABLE)?;
            let mut nodes_table = write_txn.open_table(NODES_TABLE)?;

            let mut next_id = 0;
            if let Some(last) = mapping_table.iter()?.next_back() {
                next_id = last?.0.value() + 1;
            }

            for node in nodes {
                if let Some(embedding) = &node.embedding {
                    if embedding.len() != self.dim {
                        log::warn!("Skipping node {} due to dimension mismatch", node.id_);
                        continue;
                    }

                    let node_id = node.id_.clone();
                    let emb_bytes = serde_json::to_vec(&embedding)?;
                    let node_bytes = serde_json::to_vec(&node)?;
                    
                    emb_table.insert(node_id.as_str(), emb_bytes)?;
                    nodes_table.insert(node_id.as_str(), node_bytes)?;
                    mapping_table.insert(next_id, node_id.as_str())?;
                    reverse_table.insert(node_id.as_str(), next_id)?;

                    let f32_emb: Vec<f32> = embedding.iter().map(|&x| f32::from(x)).collect();
                    pending_index_updates.push((f32_emb, next_id as usize));

                    ids.push(node_id);
                    next_id += 1;
                }
            }
        }
        
        // 2. Commit DB change
        write_txn.commit()?;

        // 3. Update HNSW index while still holding the lock
        for (emb, id) in pending_index_updates {
            hnsw.insert((&emb, id));
        }

        Ok(ids)
    }

    async fn delete(&self, node_id: &str) -> Result<()> {
        let _hnsw_lock = self.hnsw.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let write_txn = self.db.begin_write()?;
        {
            let mut emb_table = write_txn.open_table(EMBEDDINGS_TABLE)?;
            let mut nodes_table = write_txn.open_table(NODES_TABLE)?;
            let mut reverse_table = write_txn.open_table(REVERSE_MAPPING_TABLE)?;
            let mut mapping_table = write_txn.open_table(MAPPING_TABLE)?;

            let inner_id = reverse_table.get(node_id)?.map(|g| g.value());
            if let Some(id) = inner_id {
                emb_table.remove(node_id)?;
                nodes_table.remove(node_id)?;
                mapping_table.remove(id)?;
                reverse_table.remove(node_id)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn query(&self, query: VectorStoreQuery) -> Result<VectorStoreQueryResult> {
        let query_embedding = query.query_embedding.ok_or_else(|| anyhow::anyhow!("ERR_VS_QUERY_EMBEDDING_MISSING"))?;
        let f32_query: Vec<f32> = query_embedding.iter().map(|&x| f32::from(x)).collect();

        // If filters are present, we search deeper (up to multiplier * top_k) to find matches
        let search_k = if query.filters.is_some() { (query.similarity_top_k * FILTER_SEARCH_MULTIPLIER).max(MIN_SEARCH_K).min(MAX_SEARCH_K) } 
                       else { query.similarity_top_k };
        
        let neighbors = {
            let hnsw = self.hnsw.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            hnsw.search(&f32_query, search_k, 200)
        };

        let read_txn = self.db.begin_read()?;
        let mapping_table = read_txn.open_table(MAPPING_TABLE)?;
        let nodes_table = read_txn.open_table(NODES_TABLE)?;

        let mut ids = Vec::new();
        let mut similarities = Vec::new();
        let mut count = 0;

        for neighbor in neighbors {
            if count >= query.similarity_top_k { break; }

            let inner_id = neighbor.d_id as u64;
            // TOMBSTONE CHECK: If not in mapping_table, it's a ghost (deleted) entry
            if let Some(node_id) = mapping_table.get(inner_id)? {
                let node_id_str = node_id.value();
                
                if let Some(filters) = &query.filters {
                    if let Some(node_bytes) = nodes_table.get(node_id_str)? {
                        let node: Node = serde_json::from_slice(node_bytes.value().as_slice())?;
                        if !crate::vector_stores::utils::filter_metadata(&node.metadata, &filters.filters, &filters.condition) {
                            continue;
                        }
                    } else { continue; }
                }

                ids.push(node_id_str.to_string());
                similarities.push(1.0 - neighbor.distance);
                count += 1;
            }
        }

        Ok(VectorStoreQueryResult {
            nodes: None,
            similarities: Some(similarities),
            ids: Some(ids),
        })
    }

    async fn persist(&self, _path: &str) -> Result<()> {
        Ok(())
    }
}
