use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use crate::indices::vector_store::VectorStoreIndex;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryMode, MetadataFilters};

pub struct VectorIndexRetriever {
    pub index: Arc<VectorStoreIndex>,
    pub similarity_top_k: usize,
    pub filters: Option<MetadataFilters>,
}

impl VectorIndexRetriever {
    pub fn new(index: Arc<VectorStoreIndex>, similarity_top_k: usize, filters: Option<MetadataFilters>) -> Self {
        Self {
            index,
            similarity_top_k,
            filters,
        }
    }
}

#[async_trait]
impl Retriever for VectorIndexRetriever {
    async fn retrieve(&self, mut query_bundle: QueryBundle) -> Result<Vec<NodeWithScore>> {
        // 1. Ensure embedding exists
        if query_bundle.embedding.is_none() {
            let emb = self.index.embed_model.get_text_embedding(&query_bundle.query_str).await?;
            query_bundle.embedding = Some(emb);
        }

        // 2. Build Query
        let query = VectorStoreQuery {
            query_embedding: query_bundle.embedding.clone(),
            similarity_top_k: self.similarity_top_k,
            filters: self.filters.clone(),
            mode: VectorStoreQueryMode::Default,
            alpha: None,
        };

        // 3. Query Vector Store
        let query_result = self.index.vector_store.query(query).await?;

        // 4. Convert Result to NodeWithScore
        let mut node_with_scores = Vec::new();

        if let Some(nodes) = query_result.nodes {
            for (i, node) in nodes.into_iter().enumerate() {
                let score = query_result.similarities.as_ref().and_then(|sims| sims.get(i)).cloned();
                node_with_scores.push(NodeWithScore { node, score });
            }
        } else if let Some(ids) = query_result.ids {
            for (i, id) in ids.into_iter().enumerate() {
                if let Some(node) = self.index.doc_store.get_document(&id).await? {
                    let score = query_result.similarities.as_ref().and_then(|sims| sims.get(i)).cloned();
                    node_with_scores.push(NodeWithScore { node, score });
                }
            }
        }

        Ok(node_with_scores)
    }
}
