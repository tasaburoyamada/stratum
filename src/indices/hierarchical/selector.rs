use async_trait::async_trait;
use anyhow::Result;
use crate::core::schema::Node;

#[async_trait]
pub trait NodeSelector: Send + Sync {
    /// Select relevant child nodes given a parent node and a query.
    /// Returns the indices of the selected children.
    async fn select(
        &self,
        query_str: &str,
        parent_node: &Node,
        children_nodes: &[Node],
    ) -> Result<Vec<usize>>;
}
