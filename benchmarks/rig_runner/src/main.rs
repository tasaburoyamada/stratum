use rig::vector_store::in_memory_store::InMemoryVectorStore;
use rig::vector_store::VectorStoreIndex;
use rig::embeddings::EmbeddingModel;
use rig::embeddings::EmbeddingError;
use rig::embeddings::DocumentEmbeddings;
use rig::vector_store::VectorStore;
use std::time::{Instant, Duration};
use std::fs;

#[derive(Clone)]
pub struct MockEmbedding;

impl EmbeddingModel for MockEmbedding {
    const MAX_DOCUMENTS: usize = 100;
    fn ndims(&self) -> usize { 384 }

    fn embed_documents(
        &self,
        texts: impl IntoIterator<Item = String> + Send,
    ) -> impl std::future::Future<Output = Result<Vec<rig::embeddings::Embedding>, EmbeddingError>> + Send {
        let texts_vec: Vec<String> = texts.into_iter().collect();
        let count = texts_vec.len();
        async move {
            // Parallel simulation: 50ms total for the whole batch
            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut embeddings = Vec::with_capacity(count);
            for (i, _) in texts_vec.iter().enumerate() {
                embeddings.push(rig::embeddings::Embedding {
                    document: format!("doc_{}", i),
                    vec: vec![0.1; 384],
                });
            }
            Ok(embeddings)
        }
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
    let model = MockEmbedding;

    let mut docs = Vec::new();
    let data_path = if fs::metadata("benchmarks/data_large").is_ok() {
        "benchmarks/data_large"
    } else if fs::metadata("data_large").is_ok() {
        "data_large"
    } else {
        "../data_large"
    };

    let paths = fs::read_dir(data_path)?;
    for path in paths {
        let p = path?.path();
        if p.extension().map_or(false, |ext| ext == "md") {
            docs.push(fs::read_to_string(&p)?);
        }
    }

    let ingest_start = Instant::now();
    let mut store = InMemoryVectorStore::default();
    
    let batch = model.embed_documents(docs).await.unwrap();
    let doc_embs = vec![DocumentEmbeddings {
        id: "batch1".to_string(),
        embeddings: batch,
        document: serde_json::Value::String("all".to_string()),
    }];
    
    store.add_documents(doc_embs).await.unwrap();
    
    let index = store.index(model.clone());
    let ingestion_time = ingest_start.elapsed();

    let _ = index.top_n::<serde_json::Value>("Warmup", 1).await.unwrap();
    
    let query_start = Instant::now();
    let _ = index.top_n::<serde_json::Value>("What is the advantage of using Rust and PyO3 together in Stratum?", 1).await.unwrap();
    let query_latency = query_start.elapsed();

    let mem = get_mem_mb();

    println!("--- Rig Results (Parallel Mock) ---");
    println!("Ingestion Time (100 docs): {} ms", ingestion_time.as_millis());
    println!("Query Latency: {} ms", query_latency.as_millis());
    println!("Memory Usage (RSS): {:.2} MB", mem);
    println!("Total Execution: {} ms", total_start.elapsed().as_millis());
    Ok(())
}
