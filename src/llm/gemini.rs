use crate::llm::base::LlmClient;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "gemini-3.1-flash-lite".to_string()),
        }
    }
}

#[derive(Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<CandidatePart>>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: Option<String>,
}

#[async_trait]
impl LlmClient for GeminiClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let request_body = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let resp_json: GenerateContentResponse = response.json().await?;
        
        let text = resp_json
            .candidates
            .and_then(|mut c| c.pop())
            .and_then(|c| c.content)
            .and_then(|c| c.parts.and_then(|mut p| p.pop()))
            .and_then(|p| p.text)
            .unwrap_or_default();

        Ok(text)
    }

    fn stream_complete(&self, _prompt: &str) -> BoxStream<'static, Result<String>> {
        Box::pin(stream::once(async {
            Err(anyhow!("Stream not implemented for GeminiClient yet"))
        }))
    }

    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(self.clone())
    }
}
