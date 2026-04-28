use async_trait::async_trait;
use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use anyhow::Result;

#[async_trait]
pub trait ResponseSynthesizer: Send + Sync {
    async fn synthesize(&self, query_bundle: QueryBundle, nodes: Vec<NodeWithScore>) -> Result<String>;
}

pub const DEFAULT_TEXT_QA_PROMPT: &str = "Context information is below.\n---------------------\n{context_str}\n---------------------\nGiven the context information and not prior knowledge, answer the query.\nQuery: {query_str}\nAnswer: ";

pub const DEFAULT_REFINE_PROMPT: &str = "The original query is as follows: {query_str}\nWe have provided an existing answer: {existing_answer}\nWe have the opportunity to refine the existing answer (only if needed) with some more context below.\n------------\n{context_msg}\n------------\nGiven the new context, refine the original answer to better answer the query. If the context isn't useful, return the original answer.\nRefined Answer: ";
