use crate::agent::executor::{AgentExecutor, ExecutorConfig};
use crate::agent::{Context};
use crate::agent::task::Task;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoGenOutput {
    pub response: String,
}

impl From<AutoGenOutput> for Value {
    fn from(val: AutoGenOutput) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AutoGenError {
    #[error("LLM error: {0}")]
    LLMError(String),

    #[error("Orchestration error: {0}")]
    Orchestrator(String),
}

#[derive(Clone, Debug)]
pub struct AutoGenExecutor {
    pub max_turns: usize,
}

impl Default for AutoGenExecutor {
    fn default() -> Self {
        Self { max_turns: 4 }
    }
}

impl AutoGenExecutor {
    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl AgentExecutor for AutoGenExecutor {
    type Output = AutoGenOutput;
    type Error = AutoGenError;

    fn config(&self) -> ExecutorConfig {
        ExecutorConfig { max_turns: self.max_turns }
    }

    async fn execute(
        &self,
        task: &Task,
        context: Arc<Context>,
    ) -> Result<Self::Output, Self::Error> {
        let llm = context.llm();

        // Simple AutoGen-like two-role dialogue: planner -> solver -> summarizer (single pass)
        let system_prompt = format!("You are an orchestrator. Plan briefly, then request tools if needed. Task: {}", task.prompt);
        let messages = vec![autoagents_llm::chat::ChatMessage::user().content(system_prompt).build()];

        let resp = llm
            .chat(&messages, None, None)
            .await
            .map_err(|e| AutoGenError::LLMError(e.to_string()))?;
        let text = resp.text().unwrap_or_default();
        Ok(AutoGenOutput { response: text })
    }
}

