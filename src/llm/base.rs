use async_trait::async_trait;
use anyhow::Result;
use futures::stream::BoxStream;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, Result<String>>;

    fn clone_box(&self) -> Box<dyn LlmClient>;
}

