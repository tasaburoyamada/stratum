use futures::stream::BoxStream;
use stratum::core::schema::{Node, NodeWithScore};
use stratum::vector_stores::simple::SimpleVectorStore;
use stratum::vector_stores::base::VectorStore;
use stratum::vector_stores::types::{VectorStoreQuery, VectorStoreQueryMode};
use stratum::storage::docstore::simple::SimpleDocumentStore;
use stratum::storage::docstore::base::DocumentStore;
use stratum::synthesizers::compact_and_refine::CompactAndRefine;
use stratum::llm::LlmClient;
use stratum::synthesizers::base::ResponseSynthesizer;
use stratum::core::config::SynthesizerConfig;
use stratum::core::query_bundle::QueryBundle;
use std::sync::Arc;
use async_trait::async_trait;
use half::f16;
use std::time::Instant;

struct MockLlm;

#[async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        // Return the prompt itself so we can check it
        Ok(prompt.to_string())
    }

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, anyhow::Result<String>> { let prompt = prompt.to_string(); let client = self.clone_box(); use futures::stream::{self, StreamExt}; stream::once(async move { client.complete(&prompt).await }).boxed() }
    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(Self)
    }
}

#[tokio::test]
async fn test_memory_pressure_simulation() {
    let vector_store = SimpleVectorStore::new();
    let num_nodes = 10000;
    let mut nodes = Vec::new();
    
    for i in 0..num_nodes {
        let mut node = Node::new_text(format!("This is node number {}", i));
        node.embedding = Some(vec![f16::from_f32(i as f32); 384]);
        nodes.push(node);
    }

    println!("Starting insertion of {} nodes...", num_nodes);
    let start = Instant::now();
    vector_store.add(nodes).await.unwrap();
    let duration = start.elapsed();
    println!("Insertion took: {:?}", duration);

    let query = VectorStoreQuery {
        query_embedding: Some(vec![f16::from_f32(5000.0); 384]),
        similarity_top_k: 10,
        filters: None,
        mode: VectorStoreQueryMode::Default,
        alpha: None,
    };

    println!("Starting query...");
    let start = Instant::now();
    let result = vector_store.query(query).await.unwrap();
    let duration = start.elapsed();
    println!("Query took: {:?}", duration);

    assert_eq!(result.ids.unwrap().len(), 10);
}

#[tokio::test]
async fn test_token_window_edge_case_logic() {
    let config = SynthesizerConfig {
        max_tokens: 100,
        max_response_tokens: 10,
        ..Default::default()
    };
    // effective_limit = 90
    
    let synthesizer = CompactAndRefine::new(Arc::new(MockLlm), config);
    let query_bundle = QueryBundle::new("query".to_string());
    
    // Construct nodes that precisely hit boundaries
    let node1 = Node::new_text("a ".repeat(40)); 
    let node2 = Node::new_text("b ".repeat(60)); 
    
    let nodes = vec![
        NodeWithScore { node: node1, score: Some(1.0) },
        NodeWithScore { node: node2, score: Some(0.9) },
    ];
    
    let response = synthesizer.synthesize(query_bundle, nodes).await.unwrap();
    
    // Verify both nodes are included (not silently dropped)
    assert!(response.contains("a a a"));
    assert!(response.contains("b b b"));
}

#[tokio::test]
async fn test_lock_poisoning_vulnerability() {
    let doc_store = Arc::new(SimpleDocumentStore::new());
    let doc_store_clone = doc_store.clone();

    // Spawn a thread to cause poisoning
    let handle = std::thread::spawn(move || {
        doc_store_clone.force_poison();
    });

    let _ = handle.join();

    // Now try to use the doc_store
    let result = doc_store.get_document("any").await;
    
    // In current implementation, it should return an Error with "ERR_LOCK_POISONED"
    assert!(result.is_err(), "Expected error due to lock poisoning, but got {:?}", result);
    let err_msg = result.unwrap_err().to_string();
    println!("Caught expected error: {}", err_msg);
    assert!(err_msg.contains("ERR_LOCK_POISONED"));
}
