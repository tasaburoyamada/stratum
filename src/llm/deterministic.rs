use crate::llm::base::LlmClient;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use blake3;

pub struct DeterministicWrapper {
    inner: Box<dyn LlmClient>,
}

impl DeterministicWrapper {
    pub fn new(inner: Box<dyn LlmClient>) -> Self {
        Self { inner }
    }

    fn hash_request(&self, prompt: &str) -> String {
        blake3::hash(prompt.as_bytes()).to_hex().to_string()
    }
}

#[async_trait]
impl LlmClient for DeterministicWrapper {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let hash = self.hash_request(prompt);
        // Log the hash for auditability
        println!("[DETERMINISTIC_AUDIT] Request Hash: {}", hash);
        self.inner.complete(prompt).await
    }

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, Result<String>> {
        let hash = self.hash_request(prompt);
        println!("[DETERMINISTIC_AUDIT] Request Hash: {}", hash);
        self.inner.stream_complete(prompt)
    }

    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(Self {
            inner: self.inner.clone_box(),
        })
    }
}
