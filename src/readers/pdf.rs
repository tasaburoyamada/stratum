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
            // pdf-extract's extract_text is synchronous, so we run it in a blocking task
            let res = tokio::task::spawn_blocking(move || {
                pdf_extract::extract_text(&path)
                    .map_err(|e| anyhow!("PDF extraction error: {}", e))
            }).await.map_err(|e| anyhow!("Task join error: {}", e))?;

            match res {
                Ok(text) => {
                    let mut node = Node::new_text(text);
                    node.metadata.file_path = Some(path_str);
                    Ok(node)
                }
                Err(e) => Err(e),
            }
        })
        .boxed()
    }
}
