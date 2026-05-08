use rig::vector_store::in_memory_store::InMemoryVectorStore;
use rig::vector_store::{VectorStoreIndex, VectorSearchRequest};
use rig_fastembed::{Client, FastembedModel};
use rig::embeddings::EmbeddingModel;
use rig::OneOrMany;
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Doc {
    id: String,
    content: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let total_start = Instant::now();

    // 1. Setup (Measure startup)
    let startup_start = Instant::now();
    let fastembed_client = Client::new();
    let model = fastembed_client.embedding_model(&FastembedModel::AllMiniLML6V2)?;
    let startup_time = startup_start.elapsed();

    // 2. Ingestion (3 docs)
    let ingest_start = Instant::now();
    let docs = vec![
        "Stratum is a high-density RAG engine built in Rust.",
        "It uses Candle for local embeddings and redb for ACID storage.",
        "The engine is designed for AI-native data processing.",
    ];
    
    let mut entries = Vec::new();
    for (i, doc_text) in docs.into_iter().enumerate() {
        let embeddings = model.embed_texts(vec![doc_text.to_string()]).await?;
        let rig_emb = rig::embeddings::Embedding {
            document: doc_text.to_string(),
            vec: embeddings[0].vec.clone(),
        };
        entries.push((
            Doc { id: format!("doc{}", i), content: doc_text.to_string() },
            OneOrMany::one(rig_emb)
        ));
    }

    let vector_store = InMemoryVectorStore::from_documents(entries);
    let ingestion_time = ingest_start.elapsed();

    // 3. Query (Latency)
    let query_start = Instant::now();
    let index = vector_store.index(model);
    let request = VectorSearchRequest::builder()
        .query("What is Stratum?".to_string())
        .samples(1)
        .build();
    let _results = index.top_n::<Doc>(request).await?;
    let query_latency = query_start.elapsed();

    // 4. Memory
    let mem = std::fs::read_to_string("/proc/self/status")?
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .map(|l| l.to_string())
        .unwrap_or_default();

    println!("--- Rig Results (FastEmbed) ---");
    println!("Startup Time (Model Init): {:?}", startup_time);
    println!("Ingestion Time (3 docs): {:?}", ingestion_time);
    println!("Query Latency: {:?}", query_latency);
    println!("Memory Usage (RSS): {}", mem);
    println!("Total Execution: {:?}", total_start.elapsed());

    Ok(())
}
