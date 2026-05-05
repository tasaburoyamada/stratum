use crate::core::schema::{Node, NodeRelationship, RelatedNodeInfo, NodeType};
use crate::storage::index_store::IndexStruct;
use crate::storage::storage_context::StorageContext;
use crate::llm::LlmClient;
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

pub struct HierarchicalIndex {
    pub storage_context: StorageContext,
    pub llm: Arc<dyn LlmClient>,
    pub root_node_ids: Vec<String>,
    pub index_id: String,
}

impl HierarchicalIndex {
    pub fn new(
        storage_context: StorageContext,
        llm: Arc<dyn LlmClient>,
        root_node_ids: Vec<String>,
        index_id: String,
    ) -> Self {
        Self {
            storage_context,
            llm,
            root_node_ids,
            index_id,
        }
    }

    pub async fn from_nodes(
        nodes: Vec<Node>,
        storage_context: StorageContext,
        llm: Arc<dyn LlmClient>,
    ) -> Result<Self> {
        let index_id = format!("hidx_{}", blake3::hash(b"hierarchical_index"));
        
        // 1. Initial nodes are the leaves
        let mut current_level_nodes = nodes;
        let mut all_nodes = Vec::new();
        
        // Save leaves to docstore
        storage_context.docstore.add_documents(current_level_nodes.clone(), true).await?;
        all_nodes.extend(current_level_nodes.clone());

        // 2. Recursive summarization
        // For simplicity in v0.1, we group by document if available, or just chunks of 5 nodes
        while current_level_nodes.len() > 1 {
            let mut next_level_nodes = Vec::new();
            let chunks = current_level_nodes.chunks(5); // Summary of every 5 nodes

            for chunk in chunks {
                if chunk.len() == 1 && next_level_nodes.len() > 0 {
                    // Avoid single child if possible, but keep it simple for now
                }
                
                let summary_node = Self::summarize_nodes(chunk, llm.clone()).await?;
                
                // Set Relationships
                let mut rels = HashMap::new();
                let children_info: Vec<RelatedNodeInfo> = chunk.iter().map(|n| {
                    RelatedNodeInfo {
                        node_id: n.id_.clone(),
                        node_type: Some(NodeType::Text),
                        metadata: n.metadata.clone(),
                        hash: Some(n.hash()),
                    }
                }).collect();
                rels.insert(NodeRelationship::Child, children_info);
                
                let mut summary_node = summary_node;
                summary_node.relationships = rels;
                
                // Update children with Parent relationship
                for child in chunk {
                    let mut updated_child = child.clone();
                    updated_child.relationships.entry(NodeRelationship::Parent)
                        .or_insert_with(Vec::new)
                        .push(RelatedNodeInfo {
                            node_id: summary_node.id_.clone(),
                            node_type: Some(NodeType::Text),
                            metadata: summary_node.metadata.clone(),
                            hash: Some(summary_node.hash()),
                        });
                    storage_context.docstore.add_documents(vec![updated_child], true).await?;
                }

                next_level_nodes.push(summary_node);
            }
            
            storage_context.docstore.add_documents(next_level_nodes.clone(), true).await?;
            all_nodes.extend(next_level_nodes.clone());
            current_level_nodes = next_level_nodes;

            if current_level_nodes.len() <= 1 {
                break;
            }
        }

        let root_node_ids: Vec<String> = current_level_nodes.iter().map(|n| n.id_.clone()).collect();
        
        // 3. Save index struct
        let mut node_ids_dict = HashMap::new();
        for n in &all_nodes {
            node_ids_dict.insert(n.id_.clone(), n.id_.clone());
        }

        let index_struct = IndexStruct {
            index_id: index_id.clone(),
            summary: Some("Hierarchical Index (PageIndex)".to_string()),
            nodes_dict: node_ids_dict,
        };
        storage_context.index_store.add_index_struct(index_struct).await?;

        Ok(Self::new(storage_context, llm, root_node_ids, index_id))
    }

    async fn summarize_nodes(nodes: &[Node], llm: Arc<dyn LlmClient>) -> Result<Node> {
        let mut combined_text = String::new();
        for (i, node) in nodes.iter().enumerate() {
            if let crate::core::schema::NodeContent::Text(t) = &node.content {
                combined_text.push_str(&format!("Node {}:\n{}\n\n", i, t));
            }
        }

        let prompt = format!(
            "Summarize the following document chunks into a concise single paragraph that captures the main themes and key information. This summary will be used for hierarchical search.\n\n{}",
            combined_text
        );

        let summary = llm.complete(&prompt).await?;
        let mut node = Node::new_text(summary);
        node.metadata = nodes[0].metadata.clone(); // Inherit some metadata
        Ok(node)
    }
}
