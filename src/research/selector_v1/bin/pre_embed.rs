use stratum::embeddings::candle::CandleEmbedding;
use stratum::embeddings::base::Embedding;
use stratum::research::selector_v1::data_loader::SelectorDataLoader;
use candle_core::{Tensor, Device};
use hf_hub::{api::sync::Api, Repo, RepoType};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Starting Native Pre-embedding generation...");

    // 1. Download/Load Model from HuggingFace (Native .safetensors version)
    println!("Fetching model files from HuggingFace...");
    let api = Api::new()?;
    let repo = api.repo(Repo::new("sentence-transformers/all-MiniLM-L6-v2".to_string(), RepoType::Model));
    
    let model_path = repo.get("model.safetensors")?;
    let tokenizer_path = repo.get("tokenizer.json")?;
    let config_path = repo.get("config.json")?;

    println!("Loading embedding model into Candle...");
    let embed_model = Arc::new(CandleEmbedding::new(
        model_path.to_str().ok_or(anyhow!("Invalid model path"))?,
        tokenizer_path.to_str().ok_or(anyhow!("Invalid tokenizer path"))?,
        config_path.to_str().ok_or(anyhow!("Invalid config path"))?,
        None
    )?);

    // 2. Load Training Data
    let loader = SelectorDataLoader::new("selector_feeding.jsonl");
    let triplets = match loader.load_all() {
        Ok(t) => t,
        Err(e) => {
            println!("⚠️ Could not load selector_feeding.jsonl: {}. Ensure the file exists.", e);
            return Ok(());
        }
    };
    
    if triplets.is_empty() {
         println!("⚠️ No data found in selector_feeding.jsonl.");
         return Ok(());
    }

    println!("📊 Processing {} triplets...", triplets.len());

    let device = Device::Cpu;
    let mut q_tensors = Vec::new();
    let mut p_tensors = Vec::new();
    let mut c_tensors = Vec::new();
    let mut targets = Vec::new();

    for (i, triplet) in triplets.iter().enumerate() {
        let q_emb = embed_model.get_text_embedding(&triplet.query).await?;
        let p_emb = embed_model.get_text_embedding(&triplet.parent_context).await?;
        
        let q_tensor = Tensor::from_vec(q_emb.iter().map(|&x| f32::from(x)).collect::<Vec<_>>(), (1, q_emb.len()), &device)?;
        let p_tensor = Tensor::from_vec(p_emb.iter().map(|&x| f32::from(x)).collect::<Vec<_>>(), (1, p_emb.len()), &device)?;

        for (j, choice) in triplet.choices.iter().enumerate() {
            let c_emb = embed_model.get_text_embedding(choice).await?;
            let c_tensor = Tensor::from_vec(c_emb.iter().map(|&x| f32::from(x)).collect::<Vec<_>>(), (1, c_emb.len()), &device)?;
            
            let target = if triplet.selected_indices.contains(&j) { 1.0f32 } else { 0.0f32 };
            
            q_tensors.push(q_tensor.clone());
            p_tensors.push(p_tensor.clone());
            c_tensors.push(c_tensor.clone());
            targets.push(target);
        }
        println!("Processed triplet {}/{}", i + 1, triplets.len());
    }

    // 3. Batched Export
    let batched_q = Tensor::cat(&q_tensors, 0)?;
    let batched_p = Tensor::cat(&p_tensors, 0)?;
    let batched_c = Tensor::cat(&c_tensors, 0)?;
    let batched_targets = Tensor::from_vec(targets, (q_tensors.len(), 1), &device)?;

    let mut tensors_to_save: HashMap<String, Tensor> = HashMap::new();
    tensors_to_save.insert("queries".to_string(), batched_q);
    tensors_to_save.insert("parents".to_string(), batched_p);
    tensors_to_save.insert("choices".to_string(), batched_c);
    tensors_to_save.insert("targets".to_string(), batched_targets);

    let output_path = "stratum/precomputed_embeddings.safetensors";
    candle_core::safetensors::save(&tensors_to_save, output_path)?;
    println!("✅ Successfully exported {} samples to {}", q_tensors.len(), output_path);

    Ok(())
}
