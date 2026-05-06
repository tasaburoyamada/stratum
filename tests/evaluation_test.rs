use stratum::evaluation::{FaithfulnessEvaluator, RelevancyEvaluator, BaseEvaluator};
use stratum::llm::LlmClient;
use async_trait::async_trait;
use std::sync::Arc;
use futures::stream::{self, BoxStream, StreamExt};

struct MockEvalLlm;

#[async_trait]
impl LlmClient for MockEvalLlm {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        if prompt.contains("Faithfulness") {
            Ok(r#"{"score": 0.9, "feedback": "The response is mostly supported.", "passing": true}"#.to_string())
        } else {
            Ok(r#"{"score": 0.8, "feedback": "Relevant answer.", "passing": true}"#.to_string())
        }
    }

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, anyhow::Result<String>> {
        let prompt = prompt.to_string();
        let client = self.clone_box();
        stream::once(async move { client.complete(&prompt).await }).boxed()
    }

    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(Self)
    }
}

#[tokio::test]
async fn test_faithfulness_evaluator() -> anyhow::Result<()> {
    let llm = Arc::new(MockEvalLlm);
    let evaluator = FaithfulnessEvaluator::new(llm);
    
    let result = evaluator.evaluate(
        "Who is the CEO?",
        "John Doe is the CEO.",
        vec!["John Doe was appointed as CEO in 2020.".to_string()]
    ).await?;

    assert!(result.score > 0.5);
    assert!(result.passing);
    Ok(())
}

#[tokio::test]
async fn test_relevancy_evaluator() -> anyhow::Result<()> {
    let llm = Arc::new(MockEvalLlm);
    let evaluator = RelevancyEvaluator::new(llm);
    
    let result = evaluator.evaluate(
        "What is Rust?",
        "Rust is a systems programming language.",
        vec![]
    ).await?;

    assert!(result.score > 0.5);
    assert!(result.passing);
    Ok(())
}
