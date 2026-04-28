use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::Result;
use futures::stream::{self, BoxStream, StreamExt};
use walkdir::WalkDir;
use std::path::PathBuf;

use crate::core::config::ReaderConfig;

pub struct SimpleDirectoryReader {
    pub input_dir: PathBuf,
    pub recursive: bool,
    pub config: ReaderConfig,
}

impl SimpleDirectoryReader {
    pub fn new(input_dir: PathBuf, recursive: bool, config: Option<ReaderConfig>) -> Self {
        Self { 
            input_dir, 
            recursive,
            config: config.unwrap_or_default(),
        }
    }
}

impl Reader for SimpleDirectoryReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let walker = if self.recursive {
            WalkDir::new(&self.input_dir)
        } else {
            WalkDir::new(&self.input_dir).max_depth(1)
        };

        let paths: Vec<PathBuf> = walker.into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();

        let concurrency = self.config.concurrency;

        stream::iter(paths)
            .map(move |path| {
                async move {
                    let content = tokio::fs::read_to_string(&path).await?;
                    let mut node = Node::new_text(content);
                    node.metadata.insert("file_path".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
                    Ok(node)
                }
            })
            .buffer_unordered(concurrency)
            .boxed()
    }
}
