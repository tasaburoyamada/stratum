use crate::core::schema::{Node, NodeRelationship, RelatedNodeInfo, NodeType};
use crate::storage::index_store::IndexStruct;
use crate::storage::storage_context::StorageContext;
use crate::llm::LlmClient;
use crate::embeddings::base::Embedding;
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

pub struct HierarchicalIndex {
    pub storage_context: StorageContext,
    pub llm: Arc<dyn LlmClient>,
    pub embed_model: Arc<dyn Embedding>,
    pub root_node_ids: Vec<String>,
    pub index_id: String,
}

impl HierarchicalIndex {
    pub fn new(
        storage_context: StorageContext,
        llm: Arc<dyn LlmClient>,
        embed_model: Arc<dyn Embedding>,
        root_node_ids: Vec<String>,
        index_id: String,
    ) -> Self {
        Self {
            storage_context,
            llm,
            embed_model,
            root_node_ids,
            index_id,
        }
    }

    pub async fn from_storage_context_async(
        storage_context: StorageContext,
        llm: Arc<dyn LlmClient>,
        embed_model: Arc<dyn Embedding>,
        index_id: String,
    ) -> Result<Self> {
        let index_struct = storage_context.index_store.get_index_struct(&index_id).await?
            .ok_or_else(|| anyhow::anyhow!("Index {} not found in index store", index_id))?;
        
        let roots_val = index_struct.extra.get("root_node_ids")
            .ok_or_else(|| anyhow::anyhow!("root_node_ids not found in index metadata"))?;
        
        let root_node_ids: Vec<String> = serde_json::from_value(roots_val.clone())?;

        Ok(Self {
            storage_context,
            llm,
            embed_model,
            root_node_ids,
            index_id,
        })
    }

    pub async fn from_nodes(
        nodes: Vec<Node>,
        storage_context: StorageContext,
        llm: Arc<dyn LlmClient>,
        embed_model: Arc<dyn Embedding>,
    ) -> Result<Self> {
        // 1. Initial nodes are the leaves
        let mut current_level_nodes = nodes;
        let mut all_nodes = Vec::new();
        
        // Save leaves to docstore
        storage_context.docstore.add_documents(current_level_nodes.clone(), true).await?;
        all_nodes.extend(current_level_nodes.clone());

        // 2. Recursive summarization with semantic grouping
        while current_level_nodes.len() > 1 {
            use futures::stream::{self, StreamExt};
            let mut next_level_nodes = Vec::new();
            
            // Group by source (e.g. file_path) if available
            let mut groups: HashMap<String, Vec<Node>> = HashMap::new();
            for node in current_level_nodes {
                let key = node.metadata.file_path.clone()
                    .or(node.metadata.url.clone())
                    .unwrap_or_else(|| "default_group".to_string());
                groups.entry(key).or_default().push(node);
            }

            let mut tasks = Vec::new();
            for (_key, group_nodes) in groups {
                for chunk in group_nodes.chunks(5) {
                    tasks.push(chunk.to_vec());
                }
            }

            let mut summary_stream = stream::iter(tasks)
                .map(|chunk| {
                    let llm = llm.clone();
                    let storage_context = storage_context.clone();
                    async move {
                        let summary_node = Self::summarize_nodes(&chunk, llm).await?;
                        
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
                        let mut updated_children = Vec::new();
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
                            updated_children.push(updated_child);
                        }
                        storage_context.docstore.add_documents(updated_children, true).await?;

                        Ok::<Node, anyhow::Error>(summary_node)
                    }
                })
                .buffer_unordered(4); // Parallelism factor

            while let Some(res) = summary_stream.next().await {
                next_level_nodes.push(res?);
            }
            
            storage_context.docstore.add_documents(next_level_nodes.clone(), true).await?;
            all_nodes.extend(next_level_nodes.clone());
            current_level_nodes = next_level_nodes;

            if current_level_nodes.len() <= 1 {
                break;
            }
        }

        let root_node_ids: Vec<String> = current_level_nodes.iter().map(|n| n.id_.clone()).collect();
        
        // Deterministic ID generation based on sorted root node IDs
        let mut sorted_root_ids = root_node_ids.clone();
        sorted_root_ids.sort();
        let mut hasher = blake3::Hasher::new();
        for id in sorted_root_ids {
            hasher.update(id.as_bytes());
        }
        let index_id = format!("hidx_{}", hasher.finalize().to_hex().to_string().get(0..12).unwrap_or(""));
        
        // 3. Save index struct
        let mut node_ids_dict = HashMap::new();
        for n in &all_nodes {
            node_ids_dict.insert(n.id_.clone(), n.id_.clone());
        }

        let mut extra = HashMap::new();
        extra.insert("root_node_ids".to_string(), serde_json::to_value(&root_node_ids)?);

        let index_struct = IndexStruct {
            index_id: index_id.clone(),
            summary: Some("Hierarchical Index (PageIndex)".to_string()),
            nodes_dict: node_ids_dict,
            extra,
        };
        storage_context.index_store.add_index_struct(index_struct).await?;

        Ok(Self::new(storage_context, llm, embed_model, root_node_ids, index_id))
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
