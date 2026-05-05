use async_trait::async_trait;
use anyhow::Result;
use futures::stream::BoxStream;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, Result<String>> {
        // Default bridge to non-streaming
        let prompt = prompt.to_string();
        let client = self.clone_box(); // Need a way to clone for stream
        
        use futures::stream::{self, StreamExt};
        stream::once(async move {
            client.complete(&prompt).await
        }).boxed()
    }

    fn clone_box(&self) -> Box<dyn LlmClient>;
}

