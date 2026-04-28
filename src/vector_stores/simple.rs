use crate::core::schema::Node;
use crate::vector_stores::base::VectorStore;
use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryResult};
use crate::vector_stores::utils::{cosine_similarity, filter_metadata};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use half::f16;

#[derive(Serialize, Deserialize, Default)]
struct SimpleVectorStoreData {
    embedding_dict: HashMap<String, Vec<f16>>,
    metadata_dict: HashMap<String, HashMap<String, serde_json::Value>>,
}

pub struct SimpleVectorStore {
    data: RwLock<SimpleVectorStoreData>,
}

impl SimpleVectorStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(SimpleVectorStoreData::default()),
        }
    }
}

#[async_trait]
impl VectorStore for SimpleVectorStore {
    async fn add(&self, nodes: Vec<Node>) -> Result<Vec<String>> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut ids = Vec::new();

        for node in nodes {
            if let Some(embedding) = node.embedding {
                data.embedding_dict.insert(node.id_.clone(), embedding);
                data.metadata_dict.insert(node.id_.clone(), node.metadata);
                ids.push(node.id_);
            }
        }

        Ok(ids)
    }

    async fn delete(&self, node_id: &str) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        data.embedding_dict.remove(node_id);
        data.metadata_dict.remove(node_id);
        Ok(())
    }

    async fn query(&self, query: VectorStoreQuery) -> Result<VectorStoreQueryResult> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let query_embedding = query.query_embedding.ok_or_else(|| anyhow::anyhow!("Query embedding missing"))?;

        // 1. Pre-filtering
        let mut filtered_ids = Vec::new();
        let mut filtered_embeddings = Vec::new();

        for (node_id, embedding) in &data.embedding_dict {
            let metadata = data.metadata_dict.get(node_id);
            
            // Apply Metadata Filters
            let filter_passed = if let Some(filters) = &query.filters {
                if let Some(m) = metadata {
                    filter_metadata(m, &filters.filters, &filters.condition)
                } else {
                    false
                }
            } else {
                true
            };

            if filter_passed {
                filtered_ids.push(node_id.clone());
                filtered_embeddings.push(embedding);
            }
        }

        // 2. Similarity Calculation
        let mut scores: Vec<(f32, String)> = filtered_ids
            .into_iter()
            .zip(filtered_embeddings)
            .map(|(id, emb)| {
                let score = cosine_similarity(&query_embedding, emb);
                (score, id)
            })
            .collect();

        // 3. Sort and Take Top K
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let top_k = scores.into_iter().take(query.similarity_top_k).collect::<Vec<_>>();

        let mut similarities = Vec::new();
        let mut ids = Vec::new();
        for (score, id) in top_k {
            similarities.push(score);
            ids.push(id);
        }

        Ok(VectorStoreQueryResult {
            nodes: None, // Need to be fetched by Index/Retriever from DocStore if needed
            similarities: Some(similarities),
            ids: Some(ids),
        })
    }

    async fn persist(&self, path: &str) -> Result<()> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let encoded: Vec<u8> = bincode::serialize(&*data)?;
        
        let tmp_path = format!("{}.tmp", path);
        std::fs::write(&tmp_path, encoded)?;
        std::fs::rename(tmp_path, path)?;
        
        Ok(())
    }
}
