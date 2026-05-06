use crate::research::selector_v1::trainer::SelectorTrainer;
use crate::research::selector_v1::data_loader::SelectorDataLoader;
use crate::embeddings::candle::CandleEmbedding;
use anyhow::Result;
use std::sync::Arc;

/// Prototype script to run a research training session.
pub async fn run_research_v1() -> Result<()> {
    println!("🧪 Starting Stratum Selector-v1 Research...");

    // 1. Setup Environment
    // Paths are based on the common cached models in this project
    let model_dir = "lasada/.fastembed_cache/models--Qdrant--all-MiniLM-L6-v2-onnx/snapshots/5f1b8cd78bc4fb444dd171e59b18f3a3af89a079";
    let model_path = format!("{}/model.onnx", model_dir);
    let tokenizer_path = format!("{}/tokenizer.json", model_dir);
    let config_path = format!("{}/config.json", model_dir);

    let embed_model = Arc::new(CandleEmbedding::new(
        &model_path,
        &tokenizer_path,
        &config_path,
        None
    )?);

    // 2. Load Data
    let loader = SelectorDataLoader::new("selector_feeding.jsonl");
    let triplets = loader.load_all()?;
    println!("📊 Loaded {} triplets for distillation.", triplets.len());

    if triplets.is_empty() {
        println!("⚠️ No data found in selector_feeding.jsonl. Run some hierarchical queries first!");
        return Ok(());
    }

    // 3. Initialize Trainer
    let mut trainer = SelectorTrainer::new(384, embed_model)?;

    // 4. Run Training
    println!("🚀 Distilling intelligence from LLM to VectorSelector...");
    trainer.train_on_triplets(triplets).await?;
    println!("✅ Training step completed.");

    Ok(())
}
