use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::runtime::Runtime;

use ::stratum::storage::storage_context::StorageContext;
use ::stratum::indices::vector_store::VectorStoreIndex;
use ::stratum::readers::file::SimpleDirectoryReader;
use ::stratum::readers::base::Reader;
use ::stratum::node_parser::sentence_splitter::SentenceSplitter;
use ::stratum::core::ingestion::transformation::Transformation;
use ::stratum::retrievers::vector_store_retriever::VectorIndexRetriever;
use ::stratum::retrievers::base::Retriever;
use ::stratum::core::query_bundle::QueryBundle;
use ::stratum::embeddings::candle::CandleEmbedding;

#[pyclass(unsendable)]
struct Stratum {
    runtime: Runtime,
    storage_context: StorageContext,
    index: Option<Arc<VectorStoreIndex>>,
}

#[pymethods]
impl Stratum {
    #[new]
    fn new(persist_dir: Option<String>) -> PyResult<Self> {
        let rt = Runtime::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let context = if let Some(dir) = persist_dir {
            StorageContext::from_dir(&dir).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        } else {
            StorageContext::in_memory()
        };

        Ok(Self { 
            runtime: rt,
            storage_context: context,
            index: None,
        })
    }

    /// Ingest documents from a directory and build index.
    fn ingest(&mut self, path: String) -> PyResult<()> {
        self.runtime.block_on(async {
            let reader = SimpleDirectoryReader::new(
                PathBuf::from(path), 
                true, 
                Some(vec!["md".to_string(), "txt".to_string()]), 
                None
            );
            let documents = reader.load_data().await
                .map_err(|e| PyRuntimeError::new_err(format!("Reader error: {}", e)))?;
            
            let nodes = SentenceSplitter::default().transform(documents).await
                .map_err(|e| PyRuntimeError::new_err(format!("Splitter error: {}", e)))?;

            // Manually setup a default local model (all-MiniLM-L6-v2)
            // Expecting models in project root or relative to CWD
            let model_path = "models/all-MiniLM-L6-v2/model.safetensors";
            let tokenizer_path = "models/all-MiniLM-L6-v2/tokenizer.json";
            let config_path = "models/all-MiniLM-L6-v2/config.json";

            let embed_model = Arc::new(CandleEmbedding::new(
                model_path,
                tokenizer_path,
                config_path,
                None
            ).map_err(|e| PyRuntimeError::new_err(format!("Model load error: {}. Ensure models/ directory exists.", e)))?);

            let index = VectorStoreIndex::from_nodes(
                nodes,
                self.storage_context.clone(),
                embed_model,
                Default::default()
            ).await.map_err(|e| PyRuntimeError::new_err(format!("Indexing error: {}", e)))?;

            self.index = Some(Arc::new(index));
            Ok(())
        })
    }

    /// Query the index and return top-k results.
    fn query(&self, query_str: String, top_k: usize) -> PyResult<Vec<(String, f32)>> {
        let index = self.index.as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("No index found. Call ingest() first."))?;

        self.runtime.block_on(async {
            let retriever = VectorIndexRetriever::new(index.clone(), top_k, None);
            let results = retriever.retrieve(QueryBundle::new(query_str)).await
                .map_err(|e| PyRuntimeError::new_err(format!("Query error: {}", e)))?;

            let mut out = Vec::new();
            for node_with_score in results {
                let content = if let ::stratum::core::schema::NodeContent::Text(t) = &node_with_score.node.content {
                    t.clone()
                } else {
                    String::from("[Non-text content]")
                };
                out.push((content, node_with_score.score.unwrap_or(0.0)));
            }
            Ok(out)
        })
    }

    fn ping(&self) -> PyResult<String> {
        Ok("Stratum Core (Rust) is active.".to_string())
    }
}

#[pymodule]
fn stratum(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Stratum>()?;
    Ok(())
}
