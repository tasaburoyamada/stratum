
use stratum::storage::storage_context::StorageContext;
use stratum::embeddings::base::Embedding;
use stratum::indices::vector_store::VectorStoreIndex;
use stratum::node_parser::sentence_splitter::SentenceSplitter;
use stratum::core::schema::Node;
use stratum::core::ingestion::transformation::Transformation;
use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use half::f16;
use anyhow::Result;

#[derive(Debug)]
pub struct MockEmbedding;
#[async_trait]
impl Embedding for MockEmbedding {
    async fn get_text_embedding(&self, _text: &str) -> Result<Vec<f16>> {
        Ok(vec![f16::from_f32(0.1); 384])
    }
    fn model_name(&self) -> &str { "mock" }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let embed_model = Arc::new(MockEmbedding);
    
    let mut nodes = Vec::new();
    for i in 0..1000 {
        nodes.push(Node::new_text(format!("Dummy text {}", i)));
    }
    
    let splitter_start = Instant::now();
    let splitter = SentenceSplitter::default();
    let split_nodes = splitter.transform(nodes).await?;
    let splitter_time = splitter_start.elapsed();

    let storage_dir = "bench_storage_native_opt";
    if std::path::Path::new(storage_dir).exists() {
        std::fs::remove_dir_all(storage_dir)?;
    }
    std::fs::create_dir_all(storage_dir)?;
    
    let storage_context = StorageContext::from_dir(storage_dir)?;
    
    let ingest_start = Instant::now();
    let _index = Arc::new(VectorStoreIndex::from_nodes(
        split_nodes,
        storage_context.clone(),
        embed_model,
        Default::default()
    ).await?);
    let ingestion_time = ingest_start.elapsed();

    drop(_index);
    drop(storage_context); // Release redb lock

    let rebuild_start = Instant::now();
    let _storage_context_reloaded = StorageContext::from_dir(storage_dir)?;
    let rebuild_time = rebuild_start.elapsed();

    println!("--- Pure Overhead Results (1000 docs) ---");
    println!("SentenceSplitter Init & Transform: {:?}", splitter_time);
    println!("Native Storage Ingestion (Persistent): {:?}", ingestion_time);
    println!("Native Storage Index Rebuild: {:?}", rebuild_time);

    Ok(())
}

