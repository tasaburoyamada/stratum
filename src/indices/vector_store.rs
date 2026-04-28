use crate::core::schema::Node;
use crate::embeddings::base::Embedding;
use crate::vector_stores::base::VectorStore;
use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryResult};
use crate::storage::docstore::base::DocumentStore;
use anyhow::Result;
use std::sync::Arc;

use crate::core::config::IndexConfig;

pub struct VectorStoreIndex {
    pub vector_store: Arc<dyn VectorStore>,
    pub doc_store: Arc<dyn DocumentStore>,
    pub embed_model: Arc<dyn Embedding>,
    pub config: IndexConfig,
}

impl VectorStoreIndex {
    pub fn new(
        vector_store: Arc<dyn VectorStore>, 
        doc_store: Arc<dyn DocumentStore>,
        embed_model: Arc<dyn Embedding>, 
        config: IndexConfig
    ) -> Self {
        Self { vector_store, doc_store, embed_model, config }
    }

    /// Build index from nodes (equivalent to building from documents after chunking)
    pub async fn from_nodes(
        nodes: Vec<Node>,
        vector_store: Arc<dyn VectorStore>,
        doc_store: Arc<dyn DocumentStore>,
        embed_model: Arc<dyn Embedding>,
        config: IndexConfig,
    ) -> Result<Self> {
        let mut nodes_with_embeddings = nodes;
        
        // 1. Generate embeddings for all nodes
        let texts: Vec<String> = nodes_with_embeddings.iter().map(|n| {
            if let crate::core::schema::NodeContent::Text(t) = &n.content {
                t.clone()
            } else {
                String::new()
            }
        }).collect();

        // Use batch size from config
        let embeddings = embed_model.get_text_embedding_batch(texts, config.embed_batch_size).await?;

        for (node, emb) in nodes_with_embeddings.iter_mut().zip(embeddings) {
            node.embedding = Some(emb);
        }

        // 2. Add to doc store (includes embeddings for persistence if needed, 
        // but primarily to store the node content linked by ID).
        doc_store.add_documents(nodes_with_embeddings.clone(), true).await?;

        // 3. Add to vector store (consumes nodes, stores only embeddings and metadata).
        vector_store.add(nodes_with_embeddings).await?;

        Ok(Self::new(vector_store, doc_store, embed_model, config))
    }

    pub async fn query(&self, query_str: &str, top_k: usize) -> Result<VectorStoreQueryResult> {
        let query_embedding = self.embed_model.get_text_embedding(query_str).await?;
        
        let query = VectorStoreQuery {
            query_embedding: Some(query_embedding),
            similarity_top_k: top_k,
            filters: None,
            mode: crate::vector_stores::types::VectorStoreQueryMode::Default,
            alpha: None,
        };

        let mut query_result = self.vector_store.query(query).await?;

        // Fetch nodes from doc_store if not present
        if query_result.nodes.is_none() {
            if let Some(ids) = &query_result.ids {
                let mut nodes = Vec::new();
                for id in ids {
                    if let Some(node) = self.doc_store.get_document(id).await? {
                        nodes.push(node);
                    }
                }
                query_result.nodes = Some(nodes);
            }
        }

        Ok(query_result)
    }
}
