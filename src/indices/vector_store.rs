use crate::core::schema::Node;
use crate::embeddings::base::Embedding;
use crate::vector_stores::base::VectorStore;
use crate::vector_stores::types::{VectorStoreQuery, VectorStoreQueryResult};
use crate::storage::docstore::base::DocumentStore;
use crate::storage::index_store::{IndexStore, IndexStruct};
use crate::storage::storage_context::StorageContext;
use anyhow::Result;
use std::sync::Arc;

use crate::core::config::IndexConfig;

pub struct VectorStoreIndex {
    pub vector_store: Arc<dyn VectorStore>,
    pub doc_store: Arc<dyn DocumentStore>,
    pub index_store: Arc<dyn IndexStore>,
    pub embed_model: Arc<dyn Embedding>,
    pub config: IndexConfig,
    pub index_id: String,
}

impl VectorStoreIndex {
    pub fn new(
        vector_store: Arc<dyn VectorStore>, 
        doc_store: Arc<dyn DocumentStore>,
        index_store: Arc<dyn IndexStore>,
        embed_model: Arc<dyn Embedding>, 
        config: IndexConfig,
        index_id: String,
    ) -> Self {
        Self { vector_store, doc_store, index_store, embed_model, config, index_id }
    }

    pub fn from_storage_context(
        storage_context: StorageContext,
        embed_model: Arc<dyn Embedding>,
        config: IndexConfig,
        index_id: String,
    ) -> Self {
        Self {
            vector_store: storage_context.vector_store,
            doc_store: storage_context.docstore,
            index_store: storage_context.index_store,
            embed_model,
            config,
            index_id,
        }
    }

    /// Build index from nodes
    pub async fn from_nodes(
        nodes: Vec<Node>,
        storage_context: StorageContext,
        embed_model: Arc<dyn Embedding>,
        config: IndexConfig,
    ) -> Result<Self> {
        let mut nodes_with_embeddings = nodes;
        let index_id = format!("idx_{}", blake3::hash(b"vector_index")); // Deterministic ID for now
        
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

        let mut node_ids_dict = std::collections::HashMap::new();

        for (node, emb) in nodes_with_embeddings.iter_mut().zip(embeddings) {
            node.embedding = Some(emb);
            node_ids_dict.insert(node.id_.clone(), node.id_.clone()); // Simple mapping
        }

        // 2. Add to doc store
        storage_context.docstore.add_documents(nodes_with_embeddings.clone(), true).await?;

        // 3. Add to vector store
        storage_context.vector_store.add(nodes_with_embeddings).await?;

        // 4. Save to index store
        let index_struct = IndexStruct {
            index_id: index_id.clone(),
            summary: Some("Vector Store Index".to_string()),
            nodes_dict: node_ids_dict,
            extra: std::collections::HashMap::new(),
        };
        storage_context.index_store.add_index_struct(index_struct).await?;

        Ok(Self::new(
            storage_context.vector_store, 
            storage_context.docstore, 
            storage_context.index_store,
            embed_model, 
            config, 
            index_id
        ))
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
