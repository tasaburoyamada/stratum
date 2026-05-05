use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::Result;
use futures::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;

pub struct JsonReader {
    pub file_path: PathBuf,
}

impl JsonReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl Reader for JsonReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let path = self.file_path.clone();
        
        stream::once(async move {
            let content = tokio::fs::read_to_string(path).await?;
            let value: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(arr) = value.as_array() {
                // Return nodes from array
                let nodes: Vec<Result<Node>> = arr.iter().map(|v| {
                    let text = if v.is_string() {
                        v.as_str().unwrap().to_string()
                    } else {
                        v.to_string()
                    };
                    Ok(Node::new_text(text))
                }).collect();
                Ok(nodes)
            } else {
                // Return single node
                let node = Node::new_text(value.to_string());
                Ok(vec![Ok(node)])
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
