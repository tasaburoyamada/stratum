use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub score: f32, // 0.0 to 1.0
    pub feedback: String,
    pub passing: bool,
}

#[async_trait]
pub trait BaseEvaluator: Send + Sync {
    async fn evaluate(
        &self,
        query: &str,
        response: &str,
        contexts: Vec<String>,
    ) -> Result<EvaluationResult>;
}
