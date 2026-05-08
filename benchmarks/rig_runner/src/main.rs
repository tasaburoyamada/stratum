
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
        let count = texts.into_iter().count();
        async move {
            std::thread::sleep(Duration::from_millis(50 * count as u64));
            let mut embeddings = Vec::with_capacity(count);
            for i in 0..count {
                embeddings.push(rig::embeddings::Embedding {
                    document: format!("doc_{}", i),
                    vec: vec![0.1; 384],
                });
            }
            Ok(embeddings)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let total_start = Instant::now();
    let startup_start = Instant::now();
    let model = MockEmbedding;
    let startup_time = startup_start.elapsed();

    let mut docs = Vec::new();
    let paths = fs::read_dir("../data_large").or_else(|_| fs::read_dir("benchmarks/data_large"))?;
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

    let mem = std::fs::read_to_string("/proc/self/status")?.lines().find(|l| l.starts_with("VmRSS:")).unwrap_or_default().to_string();

    println!("--- Rig Results ---");
    println!("Startup Time: {:?}", startup_time);
    println!("Ingestion Time (100 docs): {:?}", ingestion_time);
    println!("Query Latency: {:?}", query_latency);
    println!("Memory Usage (RSS): {}", mem);
    println!("Total Execution: {:?}", total_start.elapsed());
    Ok(())
}

