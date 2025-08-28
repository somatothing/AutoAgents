use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QwenError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage { pub role: String, pub content: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest { pub model: String, pub messages: Vec<ChatMessage> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse { pub content: String }

#[derive(Clone)]
pub struct QwenClient { base_url: String, api_key: Option<String> }

impl QwenClient {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>) -> Self { Self { base_url: base_url.into(), api_key } }
    pub async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, QwenError> {
        let client = reqwest::Client::new();
        let mut rb = client.post(format!("{}/v1/chat/completions", self.base_url)).json(req);
        if let Some(key) = &self.api_key { rb = rb.bearer_auth(key); }
        let resp = rb.send().await?.error_for_status()?.json::<ChatResponse>().await?;
        Ok(resp)
    }
}
