use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::Result;
use futures::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;

/// A reader for JSON files that can extract structured data into Node metadata.
pub struct JsonReader {
    pub file_path: PathBuf,
}

impl JsonReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    fn value_to_node(v: &serde_json::Value) -> Node {
        let mut node = if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
            Node::new_text(text.to_string())
        } else if let Some(content) = v.get("content").and_then(|c| c.as_str()) {
            Node::new_text(content.to_string())
        } else if v.is_string() {
            Node::new_text(v.as_str().unwrap().to_string())
        } else {
            Node::new_text(v.to_string())
        };

        // Automatically map other fields to metadata
        if let Some(obj) = v.as_object() {
            for (key, val) in obj {
                if key != "text" && key != "content" {
                    node.metadata.extra.insert(key.clone(), val.clone());
                }
            }
        }
        node
    }
}

impl Reader for JsonReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let path = self.file_path.clone();
        
        stream::once(async move {
            let content = tokio::fs::read_to_string(path).await?;
            let value: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(arr) = value.as_array() {
                let nodes: Vec<Result<Node>> = arr.iter().map(|v| {
                    Ok(Self::value_to_node(v))
                }).collect();
                Ok(nodes)
            } else {
                Ok(vec![Ok(Self::value_to_node(&value))])
            }
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
