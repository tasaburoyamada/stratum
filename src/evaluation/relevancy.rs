use crate::evaluation::base::{BaseEvaluator, EvaluationResult};
use crate::llm::LlmClient;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

pub struct RelevancyEvaluator {
    llm: Arc<dyn LlmClient>,
}

impl RelevancyEvaluator {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl BaseEvaluator for RelevancyEvaluator {
    async fn evaluate(
        &self,
        query: &str,
        response: &str,
        _contexts: Vec<String>,
    ) -> Result<EvaluationResult> {
        let prompt = format!(
            "Evaluation Task: Answer Relevancy\n\n\
            User Query:\n{}\n\n\
            Response to Evaluate:\n{}\n\n\
            Criteria: Does the response directly and accurately address the user's query?\n\n\
            Respond ONLY with a valid JSON object matching this schema:\n\
            {{\n\
              \"score\": <float between 0.0 and 1.0>,\n\
              \"feedback\": \"<brief explanation>\",\n\
              \"passing\": <boolean>\n\
            }}",
            query,
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
