use stratum::storage::storage_context::StorageContext;
use stratum::indices::vector_store::VectorStoreIndex;
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
        std::thread::sleep(std::time::Duration::from_millis(50));
        Ok(vec![f16::from_f32(0.1); 384])
    }
    
    fn model_name(&self) -> &str {
        "simulated_gpu"
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let total_start = Instant::now();
    
    // 1. Setup Model
    let model_load_start = Instant::now();
    let embed_model = Arc::new(SimulatedGpuEmbedding);
    let startup_time = model_load_start.elapsed();

    // 2. Storage Setup
    let storage_dir = "bench_storage_stratum_large";
    if std::path::Path::new(storage_dir).exists() {
        std::fs::remove_dir_all(storage_dir)?;
    }
    std::fs::create_dir_all(storage_dir)?;
    let storage_context = StorageContext::in_memory();

    // 3. Ingestion (Measure Ingestion)
    println!("Reading 100 documents...");
    let reader = SimpleDirectoryReader::new(PathBuf::from("benchmarks/data_large"), true, Some(vec!["md".to_string()]), None);
    let documents = reader.load_data().await?;
    let nodes = SentenceSplitter::default().transform(documents).await?;
    
    println!("Ingesting...");
    let ingest_start = Instant::now();
    let index = Arc::new(VectorStoreIndex::from_nodes(
        nodes,
        storage_context,
        embed_model,
        Default::default()
    ).await?);
    let ingestion_time = ingest_start.elapsed();

    // 4. Query (Measure Latency)
    let retriever = VectorIndexRetriever::new(index, 1, None);
    let _ = retriever.retrieve(QueryBundle::new("Warmup".to_string())).await?;

    let query_start = Instant::now();
    let _results = retriever.retrieve(QueryBundle::new("What is the advantage of using Rust and PyO3 together in Stratum?".to_string())).await?;
    let query_latency = query_start.elapsed();

    let mem = std::fs::read_to_string("/proc/self/status")?
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .map(|l| l.to_string())
        .unwrap_or_default();

    println!("--- Stratum Results ---");
    println!("Startup Time (Model Load): {:?}", startup_time);
    println!("Ingestion Time (100 docs): {:?}", ingestion_time);
    println!("Query Latency: {:?}", query_latency);
    println!("Memory Usage (RSS): {}", mem);
    println!("Total Execution: {:?}", total_start.elapsed());

    Ok(())
}
