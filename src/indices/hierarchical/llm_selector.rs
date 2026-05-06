use crate::indices::hierarchical::selector::NodeSelector;
use crate::llm::LlmClient;
use crate::core::schema::Node;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

pub struct LlmNodeSelector {
    llm: Arc<dyn LlmClient>,
}

impl LlmNodeSelector {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl NodeSelector for LlmNodeSelector {
    async fn select(
        &self,
        query_str: &str,
        parent_node: &Node,
        children_nodes: &[Node],
    ) -> Result<Vec<usize>> {
        let mut children_text = String::new();
        for (i, node) in children_nodes.iter().enumerate() {
            if let Ok(content) = node.get_content(None) {
                children_text.push_str(&format!("Choice {}: {}\n\n", i, content));
            }
        }

        let parent_content = parent_node.get_content(None)?;

        let prompt = format!(
            "Query: {}\n\nGiven the parent context: \"{}\", which of the following choices contain information relevant to answering the query? Respond with only the choice numbers, separated by commas (e.g., 0, 2).\n\n{}",
            query_str,
            parent_content,
            children_text
        );

        let response = self.llm.complete(&prompt).await?;
        
        let mut selected_indices = Vec::new();
        for part in response.split(',') {
            if let Ok(idx) = part.trim().parse::<usize>() {
                if idx < children_nodes.len() {
                    selected_indices.push(idx);
                }
            }
        }

        // --- Selector Feeding Capture ---
        if !selected_indices.is_empty() {
            use crate::feeding::{SelectorTriplet, DatasetExporter};
            let choices_content: Vec<String> = children_nodes.iter()
                .filter_map(|n| n.get_content(None).ok())
                .collect();

            let triplet = SelectorTriplet::new(
                query_str.to_string(),
                parent_content,
                choices_content,
                selected_indices.clone()
            );
            let _ = DatasetExporter::export_jsonl(&[triplet], "selector_feeding.jsonl");
        }

        Ok(selected_indices)
    }
}
