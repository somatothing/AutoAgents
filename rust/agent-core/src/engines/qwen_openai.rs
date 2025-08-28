use super::{EngineBackend, InferenceRequest, InferenceResponse};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OpenAICompatibleEngine {
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAICompatibleEngine {
    pub fn from_env() -> anyhow::Result<Self> {
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Ok(Self {
            http: Client::new(),
            base_url,
            api_key,
            model,
        })
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolSchema<'a>>>,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ToolSchema<'a> {
    r#type: &'a str,
    function: ToolFunction<'a>,
}

#[derive(Serialize)]
struct ToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Option<String>,
}

#[async_trait]
impl EngineBackend for OpenAICompatibleEngine {
    async fn infer(&self, request: InferenceRequest) -> anyhow::Result<InferenceResponse> {
        let sys = request.system.as_deref().unwrap_or("");
        let messages = vec![
            ChatMessage { role: "system", content: sys },
            ChatMessage { role: "user", content: &request.prompt },
        ];

        let body = ChatRequest {
            model: &self.model,
            messages,
            max_tokens: request.max_tokens,
            tools: None,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("request failed")?;
        let parsed: ChatResponse = resp.json().await.context("invalid json")?;
        let text = parsed
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        Ok(InferenceResponse { output_text: text })
    }
}

