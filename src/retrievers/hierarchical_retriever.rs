use crate::core::schema::{NodeWithScore, NodeRelationship};
use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use crate::indices::hierarchical::{HierarchicalIndex, NodeSelector, LlmNodeSelector};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct HierarchicalRetriever {
    pub index: Arc<HierarchicalIndex>,
    pub selector: Arc<dyn NodeSelector>,
    pub max_traversal_depth: usize,
}

impl HierarchicalRetriever {
    pub fn new(index: Arc<HierarchicalIndex>, max_traversal_depth: usize) -> Self {
        let selector = Arc::new(LlmNodeSelector::new(index.llm.clone()));
        Self {
            index,
            selector,
            max_traversal_depth,
        }
    }

    pub fn with_selector(index: Arc<HierarchicalIndex>, selector: Arc<dyn NodeSelector>, max_traversal_depth: usize) -> Self {
        Self {
            index,
            selector,
            max_traversal_depth,
        }
    }
}

#[async_trait]
impl Retriever for HierarchicalRetriever {
    async fn retrieve(&self, query_bundle: QueryBundle) -> Result<Vec<NodeWithScore>> {
        let mut current_nodes = Vec::new();
        for id in &self.index.root_node_ids {
            if let Some(node) = self.index.storage_context.docstore.get_document(id).await? {
                current_nodes.push(node);
            }
        }

        let mut depth = 0;
        while depth < self.max_traversal_depth {
            let mut next_level_ids = Vec::new();
            let mut summary_found = false;

            for node in &current_nodes {
                if let Some(children) = node.relationships.get(&NodeRelationship::Child) {
                    summary_found = true;
                    
                    // Fetch children nodes
                    let mut children_nodes = Vec::new();
                    for child_info in children {
                        if let Some(c_node) = self.index.storage_context.docstore.get_document(&child_info.node_id).await? {
                            children_nodes.push(c_node);
                        }
                    }

                    if !children_nodes.is_empty() {
                        let selected_indices = self.selector.select(&query_bundle.query_str, node, &children_nodes).await?;
                        for idx in selected_indices {
                            if let Some(selected_node) = children_nodes.get(idx) {
                                next_level_ids.push(selected_node.id_.clone());
                            }
                        }
                    }
                } else {
                    // Leaf node, keep it
                    next_level_ids.push(node.id_.clone());
                }
            }

            if !summary_found {
                break;
            }

            // Update current nodes for next iteration
            let mut next_nodes = Vec::new();
            for id in next_level_ids {
                if let Some(node) = self.index.storage_context.docstore.get_document(&id).await? {
                    next_nodes.push(node);
                }
            }
            current_nodes = next_nodes;
            depth += 1;
        }

        let results = current_nodes.into_iter()
            .map(|node| NodeWithScore { node, score: Some(1.0) })
            .collect();

        Ok(results)
    }
}
