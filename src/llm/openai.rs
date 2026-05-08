use crate::llm::base::LlmClient;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use serde::{Deserialize, Serialize};
use reqwest::Client;

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model,
        }
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize)]
struct ChunkChoice {
    delta: ChunkDelta,
}

#[derive(Deserialize)]
struct ChunkDelta {
    content: Option<String>,
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: false,
        };

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<ChatCompletionResponse>()
            .await?;

        response.choices.first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("No response from OpenAI"))
    }

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, Result<String>> {
        let url = format!("{}/chat/completions", self.base_url);
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: true,
        };

        let client = self.client.clone();
        let api_key = self.api_key.clone();

        stream::once(async move {
            let res = client.post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&request)
                .send()
                .await
                .map_err(|e| anyhow!("Request error: {}", e))?;
            
            Ok(res.bytes_stream())
        })
        .flat_map(|res: Result<_>| {
            match res {
                Ok(bytes_stream) => {
                    bytes_stream
                        .map(|b| {
                            let b = b.map_err(|e| anyhow!("Stream error: {}", e))?;
                            let s = String::from_utf8_lossy(&b).to_string();
                            
                            // Simple SSE parsing for OpenAI
                            let mut content = String::new();
                            for line in s.lines() {
                                if line.starts_with("data: ") {
                                    let data = line.trim_start_matches("data: ");
                                    if data == "[DONE]" { break; }
                                    if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                                        if let Some(text) = &chunk.choices[0].delta.content {
                                            content.push_str(text);
                                        }
                                    }
                                }
                            }
                            Ok(content)
                        })
                        .boxed()
                }
                Err(e) => stream::once(async { Err(e) }).boxed(),
            }
        })
        .boxed()
    }

    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(Self {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
        })
    }
}
