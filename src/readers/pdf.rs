use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::{Result, anyhow};
use futures::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;

pub struct PdfReader {
    pub file_path: PathBuf,
}

impl PdfReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl Reader for PdfReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let path = self.file_path.clone();
        let path_str = path.to_string_lossy().to_string();
        
        stream::once(async move {
            // pdf-extract's extract_text_by_pages is synchronous, so we run it in a blocking task
            let res = tokio::task::spawn_blocking(move || {
                pdf_extract::extract_text_by_pages(&path)
                    .map_err(|e| anyhow!("PDF extraction error: {}", e))
            }).await.map_err(|e| anyhow!("Task join error: {}", e))?;

            match res {
                Ok(pages) => {
                    let mut nodes = Vec::new();
                    for (i, text) in pages.into_iter().enumerate() {
                        if text.trim().is_empty() {
                            continue;
                        }
                        let mut node = Node::new_text(text);
                        node.metadata.file_path = Some(path_str.clone());
                        node.metadata.extra.insert("page_label".to_string(), serde_json::Value::String((i + 1).to_string()));
                        nodes.push(Ok(node));
                    }
                    Ok(nodes)
                }
                Err(e) => Err(e),
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
