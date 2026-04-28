use async_trait::async_trait;
use crate::core::schema::Node;
use anyhow::Result;
use futures::stream::BoxStream;

#[async_trait]
pub trait Reader: Send + Sync {
    /// Load data as a stream of Nodes (Documents are specialized Nodes)
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>>;

    /// Load all data as a list (convenience method)
    async fn load_data(&self) -> Result<Vec<Node>> {
        use futures::stream::StreamExt;
        let mut nodes = Vec::new();
        let mut stream = self.lazy_load_data();
        while let Some(node_res) = stream.next().await {
            nodes.push(node_res?);
        }
        Ok(nodes)
    }
}
