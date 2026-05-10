use stratum::llm::{LlmClient, gemini::GeminiClient};
use stratum::feeding::SelectorTriplet;
use stratum::feeding::DatasetExporter;
use anyhow::{Result, anyhow};
use std::env;
use std::sync::Arc;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧬 Starting Data Synthesizer for VectorSelector (Gemini Powered)...");

    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            println!("⚠️ GEMINI_API_KEY is not set. Cannot synthesize data.");
            return Ok(());
        }
    };

    let llm: Arc<dyn LlmClient> = Arc::new(GeminiClient::new(
        api_key,
        Some("gemini-2.5-flash".to_string()),
    ));

    println!("🤖 Generating synthetic data batches...");

    let prompt = r#"
You are generating a dataset for training a hierarchical text retrieval model.
Generate 5 examples. Each example must contain:
1. 'query': A specific user question.
2. 'parent_context': A high-level summary of a document section.
3. 'choices': An array of 3 to 4 text snippets (child nodes). One or two must directly answer the query, while the others should be plausible but irrelevant.
4. 'selected_indices': An array of integers indicating which index/indices in the 'choices' array contain the answer.

Output MUST be a raw JSON array of objects. Do not include markdown code blocks.
"#;

    let mut all_triplets = Vec::new();

    // Generate 100 batches to demonstrate
    for i in 1..=100 {
        println!("⏳ Generating batch {}/100...", i);
        let response = llm.complete(prompt).await?;
        
        let raw_json = response.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```");
        
        match serde_json::from_str::<Vec<Value>>(raw_json) {
            Ok(json_array) => {
                for item in json_array {
                    let query = item["query"].as_str().unwrap_or("").to_string();
                    let parent_context = item["parent_context"].as_str().unwrap_or("").to_string();
                    let choices: Vec<String> = item["choices"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    let selected_indices: Vec<usize> = item["selected_indices"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as usize))
                        .collect();

                    if !query.is_empty() && !choices.is_empty() {
                        all_triplets.push(SelectorTriplet::new(
                            query,
                            parent_context,
                            choices,
                            selected_indices,
                        ));
                    }
                }
            },
            Err(e) => {
                println!("⚠️ Failed to parse JSON in batch {}: {}", i, e);
            }
        }
    }

    if !all_triplets.is_empty() {
        let path = "stratum/selector_feeding.jsonl";
        DatasetExporter::export_jsonl(&all_triplets, path)?;
        println!("✅ Synthesized {} new triplets and appended to {}", all_triplets.len(), path);
    } else {
        println!("⚠️ No valid triplets were generated.");
    }

    Ok(())
}
