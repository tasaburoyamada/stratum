use stratum::core::schema::Node;
use stratum::core::ingestion::pipeline::IngestionPipeline;
use stratum::core::ingestion::transformation::Transformation;
use stratum::node_parser::sentence_splitter::SentenceSplitter;
use stratum::embeddings::base::Embedding;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;
use std::sync::Arc;
use half::f16;

/// Simulated Embedding Model for benchmarking
#[derive(Debug)]
pub struct MockEmbedding {
    latency_ms: u64,
}

#[async_trait]
impl Embedding for MockEmbedding {
    async fn get_text_embedding(&self, _text: &str) -> Result<Vec<f16>> {
        if self.latency_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.latency_ms)).await;
        }
        Ok(vec![f16::from_f32(0.0); 384])
    }

    fn model_name(&self) -> &str {
        "mock_model"
    }
}

// Implement Transformation for benchmarking purpose if needed, 
// but IngestionPipeline mainly uses it for node parsing/refining.
// Here we'll just benchmark the overhead of the pipeline itself.

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Stratum Performance Benchmark ---");

    let doc_count = 100;
    let text_content = "This is a sample document for benchmarking. ".repeat(20); // Approx 200 tokens
    
    let nodes: Vec<Node> = (0..doc_count)
        .map(|i| {
            let mut node = Node::new_text(format!("Document {}: {}", i, text_content));
            node.metadata.genre = Some("Benchmark".to_string());
            node
        })
        .collect();

    // 1. Framework Overhead Benchmark (SentenceSplitter only)
    let transformations: Vec<Arc<dyn Transformation>> = vec![
        Arc::new(SentenceSplitter::default())
    ];
    let pipeline = IngestionPipeline::new(transformations, None);

    let start = Instant::now();
    let _ = pipeline.run(nodes.clone()).await?;
    let overhead = start.elapsed();
    println!("Framework Overhead (Splitter only, {} docs): {:?}", doc_count, overhead);

    // 2. Realistic Simulation (Mocking the cost of embeddings)
    // Note: IngestionPipeline doesn't run embeddings directly, 
    // it happens in VectorStoreIndex::from_nodes. 
    // We'll simulate that loop here.
    
    let mock_slow = Arc::new(MockEmbedding { latency_ms: 50 });
    
    let start = Instant::now();
    let texts: Vec<String> = nodes.iter().map(|n| n.get_content(None).unwrap()).collect();
    let _embeddings = mock_slow.get_text_embedding_batch(texts, 32).await?;
    let total_time = start.elapsed();
    
    println!("Total Ingestion Time ({} docs, 50ms latency, batch_size=32): {:?}", doc_count, total_time);
    println!("Average per document (simulated): {:?}", total_time / (doc_count as u32));

    println!("---------------------------------------");
    Ok(())
}
