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
            Respond in the following format:\n\
            Score: [0.0 to 1.0]\n\
            Feedback: [Brief explanation]\n\
            Passing: [YES/NO]",
            combined_context,
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
