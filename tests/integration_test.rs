use llama_index_rust::readers::web::WebReader;
use llama_index_rust::readers::base::Reader;
use llama_index_rust::node_parser::sentence_splitter::SentenceSplitter;
use llama_index_rust::core::ingestion::transformation::Transformation;
use llama_index_rust::embeddings::base::Embedding;
use llama_index_rust::vector_stores::simple::SimpleVectorStore;
use llama_index_rust::storage::docstore::simple::SimpleDocumentStore;
use llama_index_rust::indices::vector_store::VectorStoreIndex;
use llama_index_rust::query_engine::retriever_query_engine::RetrieverQueryEngine;
use llama_index_rust::retrievers::vector_store_retriever::VectorIndexRetriever;
use llama_index_rust::synthesizers::compact_and_refine::{CompactAndRefine, LlmClient};
use llama_index_rust::core::config::{IndexConfig, SynthesizerConfig, SplitterConfig};
use std::sync::Arc;
use async_trait::async_trait;
use half::f16;

struct MockLlm;

#[async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        Ok(format!("Mock response for prompt length: {}", prompt.len()))
    }
}

#[derive(Debug)]
struct MockEmbedding;

#[async_trait]
impl Embedding for MockEmbedding {
    async fn get_text_embedding(&self, _text: &str) -> anyhow::Result<Vec<f16>> {
        Ok(vec![f16::from_f32(0.5); 384]) // BERT-base size
    }
    fn model_name(&self) -> &str { "mock-bert" }
}

#[tokio::test]
async fn test_end_to_end_refinery() {
    // 1. Setup Reader
    let reader = WebReader::new(vec!["https://www.rust-lang.org".to_string()], None, None);
    if let Ok(documents) = reader.load_data().await {
        assert!(!documents.is_empty());

        // 2. Setup Transformation (Chunking)
        let splitter = SentenceSplitter::default();
        let nodes = splitter.transform(documents).await.unwrap();
        assert!(nodes.len() > 1);
    }
}

#[tokio::test]
async fn test_end_to_end_query() {
    use llama_index_rust::core::schema::Node;

    // 1. Create Nodes
    let nodes = vec![
        Node::new_text("Rust is a systems programming language.".to_string()),
        Node::new_text("LlamaIndex helps build RAG applications.".to_string()),
    ];

    // 2. Setup Index components
    let vector_store = Arc::new(SimpleVectorStore::new());
    let doc_store = Arc::new(SimpleDocumentStore::new());
    let embed_model = Arc::new(MockEmbedding);
    let config = IndexConfig::default();

    // 3. Build Index
    let index = Arc::new(VectorStoreIndex::from_nodes(
        nodes, 
        vector_store, 
        doc_store, 
        embed_model, 
        config
    ).await.unwrap());

    // 4. Setup Retriever & Query Engine
    let retriever = Arc::new(VectorIndexRetriever::new(index, 2, None));
    let synthesizer = Arc::new(CompactAndRefine::new(Arc::new(MockLlm), SynthesizerConfig::default()));
    let query_engine = RetrieverQueryEngine::new(retriever, synthesizer, vec![]);

    // 5. Query
    let response = query_engine.query("What is Rust?").await.unwrap();
    
    println!("Response: {}", response);
    assert!(response.contains("Mock response"));
}

#[tokio::test]
async fn test_pipeline_deduplication() {
    use llama_index_rust::core::schema::Node;
    use llama_index_rust::core::ingestion::pipeline::IngestionPipeline;

    // 1. Setup
    let doc_store = Arc::new(SimpleDocumentStore::new());
    let transformations: Vec<Arc<dyn Transformation>> = vec![
        Arc::new(SentenceSplitter::new(SplitterConfig {
            chunk_size: 10, // Small chunks
            chunk_overlap: 0,
            ..Default::default()
        }))
    ];
    let pipeline = IngestionPipeline::new(transformations, Some(doc_store.clone()));

    // 2. First Run
    let mut node = Node::new_text("This is a test document that should be split into multiple nodes.".to_string());
    // id_ is now based on content hash, so we don't need to manually set it for determinism
    let node_id = node.id_.clone();
    
    let nodes = vec![node.clone()];
    let result_nodes_1 = pipeline.run(nodes.clone()).await.unwrap();
    assert!(!result_nodes_1.is_empty());
    assert!(result_nodes_1.len() > 1);

    // 3. Second Run with same content
    let result_nodes_2 = pipeline.run(nodes).await.unwrap();
    
    // Should be skipped!
    assert!(result_nodes_2.is_empty(), "Should have been deduplicated");
}
