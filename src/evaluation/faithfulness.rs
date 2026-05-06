use crate::evaluation::base::{BaseEvaluator, EvaluationResult};
use crate::llm::LlmClient;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

pub struct FaithfulnessEvaluator {
    llm: Arc<dyn LlmClient>,
}

impl FaithfulnessEvaluator {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl BaseEvaluator for FaithfulnessEvaluator {
    async fn evaluate(
        &self,
        _query: &str,
        response: &str,
        contexts: Vec<String>,
    ) -> Result<EvaluationResult> {
        let combined_context = contexts.join("\n\n");
        let prompt = format!(
            "Evaluation Task: Faithfulness (Hallucination Detection)\n\n\
            Context:\n{}\n\n\
            Response to Evaluate:\n{}\n\n\
            Criteria: Is the response entirely supported by the provided context? Does it contain any information NOT present in the context?\n\n\
            Respond ONLY with a valid JSON object matching this schema:\n\
            {{\n\
              \"score\": <float between 0.0 and 1.0>,\n\
              \"feedback\": \"<brief explanation>\",\n\
              \"passing\": <boolean>\n\
            }}",
            combined_context,
            response
        );

        let eval_raw = self.llm.complete(&prompt).await?;
        
        let json_str = if let Some(start) = eval_raw.find('{') {
            if let Some(end) = eval_raw.rfind('}') {
                &eval_raw[start..=end]
            } else {
                &eval_raw
            }
        } else {
            &eval_raw
        };

        let result: serde_json::Value = serde_json::from_str(json_str).unwrap_or_else(|_| {
            serde_json::json!({
                "score": 0.0,
                "feedback": "Failed to parse LLM response as JSON",
                "passing": false
            })
        });

        let score = result.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let feedback = result.get("feedback").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let passing = result.get("passing").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(EvaluationResult { score, feedback, passing })
    }
}
