use stratum::storage::storage_context::StorageContext;
use stratum::indices::vector_store::VectorStoreIndex;
use stratum::core::config::IndexConfig;
use stratum::node_parser::sentence_splitter::SentenceSplitter;
use stratum::readers::file::SimpleDirectoryReader;
use stratum::retrievers::vector_store_retriever::VectorIndexRetriever;
use stratum::core::query_bundle::QueryBundle;
use stratum::retrievers::base::Retriever;
use stratum::readers::base::Reader;
use stratum::core::ingestion::transformation::Transformation;
use std::sync::Arc;
use std::time::Instant;
use std::path::PathBuf;
use async_trait::async_trait;
use stratum::embeddings::base::Embedding;
use half::f16;
use anyhow::Result;

#[derive(Debug)]
pub struct SimulatedGpuEmbedding;
#[async_trait]
impl Embedding for SimulatedGpuEmbedding {
    async fn get_text_embedding(&self, _text: &str) -> Result<Vec<f16>> {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(vec![f16::from_f32(0.1); 384])
    }
    
    fn model_name(&self) -> &str {
        "simulated_gpu"
    }
}

fn get_mem_mb() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse::<f64>().ok())
        .map(|v| v / 1024.0)
        .unwrap_or(0.0)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let total_start = Instant::now();
    
    // 1. Setup Model
    let embed_model = Arc::new(SimulatedGpuEmbedding);

    // 2. Storage Setup
    let storage_context = StorageContext::in_memory();

    // 3. Ingestion
    let reader = SimpleDirectoryReader::new(PathBuf::from("benchmarks/data_large"), true, Some(vec!["md".to_string()]), None);
    let documents = reader.load_data().await?;
    let nodes = SentenceSplitter::default().transform(documents).await?;
    
    // Use batch_size 100 for fair comparison with other "infinite parallel" mocks
    let config = IndexConfig {
        embed_batch_size: 100,
        ..Default::default()
    };

    println!("Ingesting {} nodes with batch_size=100...", nodes.len());
    let ingest_start = Instant::now();
    let index = Arc::new(VectorStoreIndex::from_nodes(
        nodes,
        storage_context,
        embed_model,
        config
    ).await?);
    let ingestion_time = ingest_start.elapsed();

    // 4. Query
    let retriever = VectorIndexRetriever::new(index, 1, None);
    let _ = retriever.retrieve(QueryBundle::new("Warmup".to_string())).await?;

    let query_start = Instant::now();
    let _results = retriever.retrieve(QueryBundle::new("What is the advantage of using Rust and PyO3 together in Stratum?".to_string())).await?;
    let query_latency = query_start.elapsed();

    let mem = get_mem_mb();

    println!("--- Stratum Results (Parallel) ---");
    println!("Ingestion Time (100 docs): {} ms", ingestion_time.as_millis());
    println!("Query Latency: {} ms", query_latency.as_millis());
    println!("Memory Usage (RSS): {:.2} MB", mem);
    println!("Total Execution: {} ms", total_start.elapsed().as_millis());

    Ok(())
}
