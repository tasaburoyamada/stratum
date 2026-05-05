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
    pub required_exts: Option<Vec<String>>,
    pub config: ReaderConfig,
}

impl SimpleDirectoryReader {
    pub fn new(input_dir: PathBuf, recursive: bool, required_exts: Option<Vec<String>>, config: Option<ReaderConfig>) -> Self {
        Self { 
            input_dir, 
            recursive,
            required_exts,
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

        let required_exts = self.required_exts.clone();

        let paths: Vec<PathBuf> = walker.into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(move |e| {
                if let Some(exts) = &required_exts {
                    if let Some(ext) = e.path().extension() {
                        exts.contains(&ext.to_string_lossy().to_string())
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        let concurrency = self.config.concurrency;

        stream::iter(paths)
            .map(move |path| {
                async move {
                    let content = tokio::fs::read_to_string(&path).await?;
                    let mut node = Node::new_text(content);
                    node.metadata.file_path = Some(path.to_string_lossy().to_string());
                    Ok(node)
                }
            })
            .buffer_unordered(concurrency)
            .boxed()
    }
}
