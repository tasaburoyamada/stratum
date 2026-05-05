use crate::core::schema::Node;
use crate::vector_stores::base::VectorStore;
use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryResult};
use anyhow::Result;
use async_trait::async_trait;
use hnsw_rs::prelude::*;
use redb::{Database, TableDefinition, ReadableTable};
use std::sync::{Arc, RwLock};

const NODES_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("nodes");
const MAPPING_TABLE: TableDefinition<u64, &str> = TableDefinition::new("mapping");
const REVERSE_MAPPING_TABLE: TableDefinition<&str, u64> = TableDefinition::new("reverse_mapping");

pub struct NativeVectorStore {
    db: Database,
    hnsw: Arc<RwLock<Hnsw<'static, f32, DistCosine>>>,
    dim: usize,
}

impl NativeVectorStore {
    pub fn new(path: &str, dim: usize) -> Result<Self> {
        let db = Database::create(path)?;

        // Initialize tables
        {
            let write_txn = db.begin_write()?;
            {
                let _ = write_txn.open_table(NODES_TABLE)?;
                let _ = write_txn.open_table(MAPPING_TABLE)?;
                let _ = write_txn.open_table(REVERSE_MAPPING_TABLE)?;
            }
            write_txn.commit()?;
        }

        let hnsw = Hnsw::new(16, 100, 16, 200, DistCosine);
        let hnsw = Arc::new(RwLock::new(hnsw));

        let store = Self { db, hnsw, dim };
        store.rebuild_index()?;

        Ok(store)
    }

    fn rebuild_index(&self) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let nodes_table = read_txn.open_table(NODES_TABLE)?;
        let reverse_table = read_txn.open_table(REVERSE_MAPPING_TABLE)?;
        let hnsw = self.hnsw.write().unwrap();

        for result in nodes_table.iter()? {
            let (id, node_bytes) = result?;
            let node: Node = bincode::deserialize(node_bytes.value().as_slice())?;
            if let Some(embedding) = node.embedding {
                let f32_emb: Vec<f32> = embedding.iter().map(|&x| f32::from(x)).collect();
                
                if let Some(inner_id) = reverse_table.get(id.value())? {
                    hnsw.insert((&f32_emb, inner_id.value() as usize));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl VectorStore for NativeVectorStore {
    async fn add(&self, nodes: Vec<Node>) -> Result<Vec<String>> {
        let write_txn = self.db.begin_write()?;
        let mut ids = Vec::new();

        {
            let mut nodes_table = write_txn.open_table(NODES_TABLE)?;
            let mut mapping_table = write_txn.open_table(MAPPING_TABLE)?;
            let mut reverse_table = write_txn.open_table(REVERSE_MAPPING_TABLE)?;
            let hnsw = self.hnsw.write().unwrap();

            // Find current max ID for auto-increment
            let mut next_id = 0;
            if let Some(last) = mapping_table.iter()?.rev().next() {
                next_id = last?.0.value() + 1;
            }

            for node in nodes {
                if let Some(embedding) = &node.embedding {
                    if embedding.len() != self.dim {
                        continue;
                    }

                    let node_id = node.id_.clone();
                    let node_bytes = bincode::serialize(&node)?;
                    
                    nodes_table.insert(node_id.as_str(), node_bytes)?;
                    mapping_table.insert(next_id, node_id.as_str())?;
                    reverse_table.insert(node_id.as_str(), next_id)?;

                    let f32_emb: Vec<f32> = embedding.iter().map(|&x| f32::from(x)).collect();
                    hnsw.insert((&f32_emb, next_id as usize));

                    ids.push(node_id);
                    next_id += 1;
                }
            }
        }
        write_txn.commit()?;

        Ok(ids)
    }

    async fn delete(&self, node_id: &str) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut nodes_table = write_txn.open_table(NODES_TABLE)?;
            let mut reverse_table = write_txn.open_table(REVERSE_MAPPING_TABLE)?;
            let mut mapping_table = write_txn.open_table(MAPPING_TABLE)?;

            let inner_id = reverse_table.get(node_id)?.map(|g| g.value());
            if let Some(id) = inner_id {
                nodes_table.remove(node_id)?;
                mapping_table.remove(id)?;
                reverse_table.remove(node_id)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    async fn query(&self, query: VectorStoreQuery) -> Result<VectorStoreQueryResult> {
        let query_embedding = query.query_embedding.ok_or_else(|| anyhow::anyhow!("Query embedding missing"))?;
        let f32_query: Vec<f32> = query_embedding.iter().map(|&x| f32::from(x)).collect();

        // 1. Vector Search (HNSW)
        // We take more than top_k to allow for filtering
        let search_k = if query.filters.is_some() {
            query.similarity_top_k * 5
        } else {
            query.similarity_top_k
        };

        let hnsw = self.hnsw.read().unwrap();
        let neighbors = hnsw.search(&f32_query, search_k, 200);

        let read_txn = self.db.begin_read()?;
        let mapping_table = read_txn.open_table(MAPPING_TABLE)?;
        let nodes_table = read_txn.open_table(NODES_TABLE)?;

        let mut ids = Vec::new();
        let mut similarities = Vec::new();
        let mut count = 0;

        for neighbor in neighbors {
            if count >= query.similarity_top_k {
                break;
            }

            let inner_id = neighbor.d_id as u64;
            if let Some(node_id) = mapping_table.get(inner_id)? {
                let node_id_str = node_id.value();
                
                // Apply Filters
                if let Some(filters) = &query.filters {
                    if let Some(node_bytes) = nodes_table.get(node_id_str)? {
                        let node: Node = bincode::deserialize(node_bytes.value().as_slice())?;
                        if !crate::vector_stores::utils::filter_metadata(&node.metadata, &filters.filters, &filters.condition) {
                            continue;
                        }
                    } else {
                        continue;
                    }
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
