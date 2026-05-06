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
            Respond in the following format:\n\
            Score: [0.0 to 1.0]\n\
            Feedback: [Brief explanation]\n\
            Passing: [YES/NO]",
            query,
            response
        );

        let eval_raw = self.llm.complete(&prompt).await?;
        
        // Simple parsing
        let mut score = 0.0;
        let mut feedback = String::new();
        let mut passing = false;

        for line in eval_raw.lines() {
            if line.starts_with("Score:") {
                score = line.replace("Score:", "").trim().parse().unwrap_or(0.0);
            } else if line.starts_with("Feedback:") {
                feedback = line.replace("Feedback:", "").trim().to_string();
            } else if line.starts_with("Passing:") {
                passing = line.to_uppercase().contains("YES");
            }
        }

        Ok(EvaluationResult { score, feedback, passing })
    }
}
