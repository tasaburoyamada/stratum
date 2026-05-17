use stratum::research::selector_v1::trainer::SelectorTrainer;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use anyhow::Result;

fn main() -> Result<()> {
    let dim = 768; 
    let mut trainer = SelectorTrainer::new(dim)?;
    
    println!("🚀 Starting training process...");
    
    let file = File::open("stratum/selector_feeding.jsonl")?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        let _v: Value = serde_json::from_str(&line)?;
        // Embedding/Training logic to be implemented.
    }
    
    trainer.save("stratum/src/research/selector_v1/selector_weights.safetensors")?;
    Ok(())
}
