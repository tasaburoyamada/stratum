use crate::research::selector_v1::trainer::SelectorTrainer;
use candle_core::{safetensors::load, Device};
use anyhow::{Result, anyhow};

/// Prototype script to run a research training session using pre-computed embeddings.
pub async fn run_research_v1() -> Result<()> {
    println!("🧪 Starting Stratum Selector-v1 Research (High-Speed Batch Mode)...");

    let device = Device::Cpu;
    let data_path = "stratum/precomputed_embeddings.safetensors";

    // 1. Load Pre-computed Tensors
    println!("Loading pre-computed embeddings from {}...", data_path);
    let tensors = match load(data_path, &device) {
        Ok(t) => t,
        Err(_) => {
            println!("⚠️ Could not load {}. Please run `cargo run --bin pre_embed` first.", data_path);
            return Ok(());
        }
    };

    let queries = tensors.get("queries").ok_or(anyhow!("Missing 'queries' tensor"))?;
    let parents = tensors.get("parents").ok_or(anyhow!("Missing 'parents' tensor"))?;
    let choices = tensors.get("choices").ok_or(anyhow!("Missing 'choices' tensor"))?;
    let targets = tensors.get("targets").ok_or(anyhow!("Missing 'targets' tensor"))?;

    println!("📊 Loaded batch of {} samples.", queries.dim(0)?);

    // 2. Initialize Trainer
    // The embedding dimension is 384 (all-MiniLM-L6-v2)
    let mut trainer = SelectorTrainer::new(384)?;

    // 3. Run Training (e.g., 50 epochs)
    println!("🚀 Distilling intelligence...");
    trainer.train_on_tensors(queries, choices, parents, targets, 50)?;
    println!("✅ Training step completed.");

    // 4. Save Model
    let save_path = "stratum/src/research/selector_v1/selector_weights.safetensors";
    trainer.save(save_path)?;

    Ok(())
}
