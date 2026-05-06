use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::Result;
use futures::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;

/// A specialized reader for Salesforce exam data that optimizes the structure for RAG.
pub struct SalesforceReader {
    pub file_path: PathBuf,
}

impl SalesforceReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    fn value_to_node(v: &serde_json::Value) -> Node {
        let question = v.get("question").and_then(|q| q.as_str()).unwrap_or("");
        let mut rich_text = format!("QUESTION: {}\n\nOPTIONS:\n", question);

        if let Some(options) = v.get("options").and_then(|o| o.as_object()) {
            for (key, val) in options {
                if let Some(opt_text) = val.as_str() {
                    rich_text.push_str(&format!("{}: {}\n", key, opt_text));
                }
            }
        }

        if let Some(explanation) = v.get("explanation").and_then(|e| e.as_str()) {
            rich_text.push_str(&format!("\nEXPLANATION: {}", explanation));
        }

        let mut node = Node::new_text(rich_text);

        // Map all fields to metadata for filtering/tracking
        if let Some(obj) = v.as_object() {
            for (key, val) in obj {
                node.metadata.extra.insert(key.clone(), val.clone());
            }
        }
        node
    }
}

impl Reader for SalesforceReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let path = self.file_path.clone();
        
        stream::once(async move {
            let content = tokio::fs::read_to_string(path).await?;
            
            // Try to parse as JSONL first, then as a single JSON array
            let mut nodes = Vec::new();
            
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = value.as_array() {
                    for v in arr {
                        nodes.push(Ok(Self::value_to_node(v)));
                    }
                } else {
                    nodes.push(Ok(Self::value_to_node(&value)));
                }
            } else {
                // Try JSONL parsing
                for line in content.lines() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        nodes.push(Ok(Self::value_to_node(&v)));
                    }
                }
            }
            Ok(nodes)
        })
        .flat_map(|res| {
            match res {
                Ok(nodes) => stream::iter(nodes).boxed(),
                Err(e) => stream::once(async { Err(e) }).boxed(),
            }
        })
        .boxed()
    }
}
