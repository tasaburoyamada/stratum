use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::Result;
use futures::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;

/// A reader for Markdown files that splits by headers to preserve structure.
pub struct MarkdownReader {
    pub file_path: PathBuf,
}

impl MarkdownReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl Reader for MarkdownReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let path = self.file_path.clone();
        let path_str = path.to_string_lossy().to_string();
        
        stream::once(async move {
            let content = tokio::fs::read_to_string(path).await?;
            let mut nodes = Vec::new();
            
            // Simple split by Markdown headers (# , ## , etc.)
            let mut current_section = String::new();
            let mut current_header = String::new();
            
            for line in content.lines() {
                if line.starts_with("#") {
                    // Save previous section if not empty
                    if !current_section.trim().is_empty() {
                        let mut node = Node::new_text(current_section.clone());
                        node.metadata.file_path = Some(path_str.clone());
                        node.metadata.extra.insert("header".to_string(), serde_json::json!(current_header.clone()));
                        nodes.push(Ok(node));
                    }
                    current_header = line.to_string();
                    current_section = line.to_string() + "\n";
                } else {
                    current_section.push_str(line);
                    current_section.push('\n');
                }
            }
            
            // Last section
            if !current_section.trim().is_empty() {
                let mut node = Node::new_text(current_section);
                node.metadata.file_path = Some(path_str);
                node.metadata.extra.insert("header".to_string(), serde_json::json!(current_header));
                nodes.push(Ok(node));
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
