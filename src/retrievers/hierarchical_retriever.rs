use crate::core::schema::{Node, NodeWithScore, NodeRelationship};
use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use crate::indices::hierarchical::HierarchicalIndex;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct HierarchicalRetriever {
    pub index: Arc<HierarchicalIndex>,
    pub max_traversal_depth: usize,
}

impl HierarchicalRetriever {
    pub fn new(index: Arc<HierarchicalIndex>, max_traversal_depth: usize) -> Self {
        Self {
            index,
            max_traversal_depth,
        }
    }

    async fn select_relevant_children(&self, parent_node: &Node, query_str: &str) -> Result<Vec<String>> {
        let children = parent_node.relationships.get(&NodeRelationship::Child);
        if children.is_none() || children.unwrap().is_empty() {
            return Ok(Vec::new());
        }
        let children = children.unwrap();

        let mut children_text = String::new();
        let mut choices_content = Vec::new();
        for (i, child) in children.iter().enumerate() {
            // We need to fetch the child node to see its content
            if let Some(node) = self.index.storage_context.docstore.get_document(&child.node_id).await? {
                if let Ok(content) = node.get_content(None) {
                    children_text.push_str(&format!("Choice {}: {}\n\n", i, content));
                    choices_content.push(content);
                }
            }
        }

        let parent_content = parent_node.get_content(None)?;

        let prompt = format!(
            "Query: {}\n\nGiven the parent context: \"{}\", which of the following choices contain information relevant to answering the query? Respond with only the choice numbers, separated by commas (e.g., 0, 2).\n\n{}",
            query_str,
            parent_content,
            children_text
        );

        let response = self.index.llm.complete(&prompt).await?;
        
        // Simple parsing of choice numbers
        let mut relevant_ids = Vec::new();
        let mut selected_indices = Vec::new();
        for part in response.split(',') {
            if let Ok(idx) = part.trim().parse::<usize>() {
                if idx < children.len() {
                    relevant_ids.push(children[idx].node_id.clone());
                    selected_indices.push(idx);
                }
            }
        }

        // --- Selector Feeding Capture ---
        if !selected_indices.is_empty() {
            use crate::feeding::{SelectorTriplet, DatasetExporter};
            let triplet = SelectorTriplet::new(
                query_str.to_string(),
                parent_content,
                choices_content,
                selected_indices
            );
            // Append to a specific feeding file for selectors
            let _ = DatasetExporter::export_jsonl(&[triplet], "selector_feeding.jsonl");
        }

        Ok(relevant_ids)
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
                if node.relationships.contains_key(&NodeRelationship::Child) {
                    summary_found = true;
                    let relevant_children = self.select_relevant_children(node, &query_bundle.query_str).await?;
                    next_level_ids.extend(relevant_children);
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
            .map(|node| NodeWithScore { node, score: Some(1.0) }) // Scores are binary in this model for now
            .collect();

        Ok(results)
    }
}
