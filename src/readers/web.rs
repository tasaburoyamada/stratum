use crate::core::schema::Node;
use crate::readers::base::Reader;
use anyhow::{Result, anyhow};
use futures::stream::{self, BoxStream, StreamExt};
use scraper::{Html, Selector};

use crate::core::config::WebConfig;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

pub struct WebReader {
    pub urls: Vec<String>,
    pub selector: String,
    pub config: WebConfig,
}

impl WebReader {
    pub fn new(urls: Vec<String>, selector: Option<String>, config: Option<WebConfig>) -> Self {
        Self { 
            urls, 
            selector: selector.unwrap_or_else(|| "body".to_string()),
            config: config.unwrap_or_default(),
        }
    }

    async fn fetch_url(url: String, selector_str: String, user_agent: String) -> Result<Node> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .build()?;

        let response = client.get(&url).send().await?;
        let html_content = response.text().await?;
        
        let document = Html::parse_document(&html_content);
        let body_selector = Selector::parse(&selector_str).map_err(|e| anyhow!("Selector error: {:?}", e))?;
        
        let mut markdown = String::new();
        if let Some(element) = document.select(&body_selector).next() {
            let html = element.html();
            markdown = html2md::parse_html(&html);
        }

        let mut node = Node::new_text(markdown);
        node.metadata.url = Some(url);
        Ok(node)
    }
}

impl Reader for WebReader {
    fn lazy_load_data(&self) -> BoxStream<'static, Result<Node>> {
        let urls = self.urls.clone();
        let selector = self.selector.clone();
        let config = self.config.clone();
        let concurrency = self.config.concurrency;

        stream::iter(urls)
            .map(move |url| {
                let s = selector.clone();
                let c = config.clone();
                async move {
                    // Apply delay and jitter
                    if c.delay_ms > 0 {
                        let jitter = if c.jitter_ms > 0 {
                            rand::thread_rng().gen_range(0..c.jitter_ms)
                        } else {
                            0
                        };
                        sleep(Duration::from_millis(c.delay_ms + jitter)).await;
                    }
                    Self::fetch_url(url, s, c.user_agent).await
                }
            })
            .buffer_unordered(concurrency) // Concurrency control
            .boxed()
    }
}
