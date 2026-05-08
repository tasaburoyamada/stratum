use stratum::storage::storage_context::StorageContext;
use stratum::embeddings::candle::CandleEmbedding;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let total_start = Instant::now();
    
    // 1. Setup Model (Measure startup)
    let model_dir = "models/all-MiniLM-L6-v2";
    println!("Loading model from {}...", model_dir);
    let model_load_start = Instant::now();
    let embed_model = Arc::new(CandleEmbedding::new(
        &format!("{}/model.safetensors", model_dir),
        &format!("{}/tokenizer.json", model_dir),
        &format!("{}/config.json", model_dir),
        None
    )?);
    let startup_time = model_load_start.elapsed();
    println!("Model loaded.");

    // 2. Storage Setup
    println!("Setting up storage...");
    let storage_dir = "bench_storage_stratum";
    if std::path::Path::new(storage_dir).exists() {
        std::fs::remove_dir_all(storage_dir)?;
    }
    std::fs::create_dir_all(storage_dir)?;
    let storage_context = StorageContext::from_dir(storage_dir)?;
    println!("Storage setup.");

    // 3. Ingestion (Measure Ingestion)
    let ingest_start = Instant::now();
    let reader = SimpleDirectoryReader::new(PathBuf::from("benchmarks/data"), true, Some(vec!["md".to_string()]), None);
    let documents = reader.load_data().await?;
    let nodes = SentenceSplitter::default().transform(documents).await?;
    
    let index = Arc::new(VectorStoreIndex::from_nodes(
        nodes,
        storage_context,
        embed_model,
        Default::default()
    ).await?);
    let ingestion_time = ingest_start.elapsed();

    // 4. Query (Measure Latency)
    let retriever = VectorIndexRetriever::new(index, 1, None);
    let query_start = Instant::now();
    let _results = retriever.retrieve(QueryBundle::new("What is Stratum?".to_string())).await?;
    let query_latency = query_start.elapsed();

    // 5. Memory
    let mem = std::fs::read_to_string("/proc/self/status")?
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .map(|l| l.to_string())
        .unwrap_or_default();

    println!("--- Stratum Results ---");
    println!("Startup Time (Model Load): {:?}", startup_time);
    println!("Ingestion Time (3 docs): {:?}", ingestion_time);
    println!("Query Latency: {:?}", query_latency);
    println!("Memory Usage (RSS): {}", mem);
    println!("Total Execution: {:?}", total_start.elapsed());

    Ok(())
}
