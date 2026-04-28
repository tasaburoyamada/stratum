use async_trait::async_trait;
use anyhow::Result;
use std::fmt::Debug;
use half::f16;
use futures::stream::{self, StreamExt};

#[async_trait]
pub trait Embedding: Send + Sync + Debug {
    /// Embed a single text
    async fn get_text_embedding(&self, text: &str) -> Result<Vec<f16>>;

    /// Embed a batch of texts with concurrency control
    async fn get_text_embedding_batch(&self, texts: Vec<String>, batch_size: usize) -> Result<Vec<Vec<f16>>> {
        let results = stream::iter(texts)
            .map(|text| async move {
                self.get_text_embedding(&text).await
            })
            .buffered(batch_size)
            .collect::<Vec<_>>()
            .await;

        let mut final_results = Vec::with_capacity(results.len());
        for res in results {
            final_results.push(res?);
        }
        Ok(final_results)
    }

    fn model_name(&self) -> &str;
}
