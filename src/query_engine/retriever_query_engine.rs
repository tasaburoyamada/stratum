use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use crate::postprocessors::base::NodePostprocessor;
use crate::synthesizers::base::ResponseSynthesizer;
use anyhow::Result;
use std::sync::Arc;

pub struct RetrieverQueryEngine {
    pub retriever: Arc<dyn Retriever>,
    pub node_postprocessors: Vec<Arc<dyn NodePostprocessor>>,
    pub response_synthesizer: Arc<dyn ResponseSynthesizer>,
}

impl RetrieverQueryEngine {
    pub fn new(
        retriever: Arc<dyn Retriever>,
        response_synthesizer: Arc<dyn ResponseSynthesizer>,
        node_postprocessors: Vec<Arc<dyn NodePostprocessor>>,
    ) -> Self {
        Self {
            retriever,
            response_synthesizer,
            node_postprocessors,
        }
    }

    pub async fn query(&self, query_str: &str) -> Result<String> {
        let query_bundle = QueryBundle::new(query_str.to_string());
        
        // 1. Retrieve
        let mut nodes = self.retriever.retrieve(query_bundle.clone()).await?;

        // 2. Postprocess
        for postprocessor in &self.node_postprocessors {
            nodes = postprocessor.postprocess_nodes(nodes, &query_bundle).await?;
        }

        // 3. Synthesize
        self.response_synthesizer.synthesize(query_bundle, nodes).await
    }
}
