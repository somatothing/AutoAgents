use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub system: Option<String>,
    pub tools: Vec<ToolCall>,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub output_text: String,
}

#[async_trait]
pub trait EngineBackend: Send + Sync {
    async fn infer(&self, request: InferenceRequest) -> anyhow::Result<InferenceResponse>;
}

#[cfg(feature = "burn")]
pub mod burn_backend;

pub mod qwen_openai;

