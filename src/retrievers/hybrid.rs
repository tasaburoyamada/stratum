use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use crate::postprocessors::vlog_bias::VlogBiasPostprocessor;
use crate::postprocessors::base::NodePostprocessor;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct HybridRetriever {
    pub vector_retriever: Box<dyn Retriever>,
    pub bias_postprocessor: Arc<VlogBiasPostprocessor>,
}

#[async_trait]
impl Retriever for HybridRetriever {
    async fn retrieve(&self, query_bundle: QueryBundle) -> Result<Vec<NodeWithScore>> {
        let nodes = self.vector_retriever.retrieve(query_bundle.clone()).await?;
        self.bias_postprocessor.postprocess_nodes(nodes, &query_bundle).await
    }
}
