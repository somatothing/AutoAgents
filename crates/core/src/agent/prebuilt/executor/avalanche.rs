use crate::agent::executor::{AgentExecutor, ExecutorConfig};
use crate::agent::{Context};
use crate::agent::task::Task;
use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvalancheOutput {
    pub responses: Vec<String>,
    pub merged: String,
}

impl From<AvalancheOutput> for Value {
    fn from(val: AvalancheOutput) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AvalancheError {
    #[error("LLM error: {0}")]
    LLMError(String),
}

#[derive(Clone, Debug)]
pub struct AvalancheExecutor {
    pub branches: usize,
}

impl Default for AvalancheExecutor {
    fn default() -> Self {
        Self { branches: 3 }
    }
}

impl AvalancheExecutor {
    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl AgentExecutor for AvalancheExecutor {
    type Output = AvalancheOutput;
    type Error = AvalancheError;

    fn config(&self) -> ExecutorConfig {
        ExecutorConfig { max_turns: 1 }
    }

    async fn execute(
        &self,
        task: &Task,
        context: Arc<Context>,
    ) -> Result<Self::Output, Self::Error> {
        let llm = context.llm();

        let prompts: Vec<String> = (0..self.branches)
            .map(|i| format!("Branch {i}: propose a distinct approach to solve: {}", task.prompt))
            .collect();

        let mut responses = Vec::new();
        for p in prompts.into_iter() {
            let msgs = vec![autoagents_llm::chat::ChatMessage::user().content(p).build()];
            let r = llm.chat(&msgs, None, None).await;
            match r {
                Ok(cr) => responses.push(cr.text().unwrap_or_default()),
                Err(e) => responses.push(format!("error: {}", e)),
            }
        }

        // Merge step
        let merge_prompt = format!(
            "Merge the following proposals into a concise plan:\n{}",
            responses
                .iter()
                .enumerate()
                .map(|(i, r)| format!("[{i}] {r}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let merge_msgs = vec![autoagents_llm::chat::ChatMessage::user().content(merge_prompt).build()];
        let merged = llm
            .chat(&merge_msgs, None, None)
            .await
            .map_err(|e| AvalancheError::LLMError(e.to_string()))?
            .text()
            .unwrap_or_default();

        Ok(AvalancheOutput { responses, merged })
    }
}

