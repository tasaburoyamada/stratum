use stratum::readers::file::SimpleDirectoryReader;
use stratum::readers::base::Reader;
use stratum::node_parser::sentence_splitter::SentenceSplitter;
use stratum::core::ingestion::transformation::Transformation;
use stratum::embeddings::candle::CandleEmbedding;
use stratum::indices::vector_store::VectorStoreIndex;
use stratum::retrievers::base::Retriever;
use stratum::retrievers::vector_store_retriever::VectorIndexRetriever;
use stratum::storage::storage_context::StorageContext;
use stratum::core::config::IndexConfig;
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Stratum Standalone RAG Demo");

    // 1. Setup Model Paths
    let model_dir = "lasada/.fastembed_cache/models--Qdrant--all-MiniLM-L6-v2-onnx/snapshots/5f1b8cd78bc4fb444dd171e59b18f3a3af89a079";
    let embed_model = Arc::new(CandleEmbedding::new(
        &format!("{}/model.onnx", model_dir),
        &format!("{}/tokenizer.json", model_dir),
        &format!("{}/config.json", model_dir),
        None
    )?);
    
    // 2. Load and Process Data
    let storage_dir = "demo_storage";
    let storage_context = StorageContext::from_dir(storage_dir)?;
    
    let input_path = PathBuf::from("stratum/src");
    let reader = SimpleDirectoryReader::new(input_path, true, Some(vec!["rs".to_string()]), None);
    
    println!("📖 Loading source code from stratum/src...");
    let documents = reader.load_data().await?;
    let nodes = SentenceSplitter::default().transform(documents).await?;
    println!("✅ Processed {} nodes.", nodes.len());

    // 3. Build/Restore Index
    println!("🏗️ Building Vector Index (ACID-backed)...");
    let index = Arc::new(VectorStoreIndex::from_nodes(
        nodes,
        storage_context.clone(),
        embed_model,
        IndexConfig::default()
    ).await?);

    // 4. Query
    let query = "How is NativeVectorStore implemented?";
    println!("🔍 Querying: \"{}\"", query);

    let retriever = VectorIndexRetriever::new(index, 3, None);
    
    let results = retriever.retrieve(stratum::core::query_bundle::QueryBundle::new(query.to_string())).await?;
    
    for (i, res) in results.iter().enumerate() {
        println!("\n[Result {} - Score: {:?}]", i + 1, res.score);
        if let stratum::core::schema::NodeContent::Text(t) = &res.node.content {
            println!("{}", t.chars().take(200).collect::<String>());
        }
    }

    println!("\n✨ Demo completed successfully.");
    Ok(())
}
