use crate::{InMemoryVectorStore, RagEngine};
use autoagents_core::core::protocol::{ActorID, Event, SubmissionId, TaskResult};
use autoagents_llm::chat::ChatMessage;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatLogEntry {
    pub role: String,
    pub message_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEventEntry {
    pub id: String,
    pub tool_name: String,
    pub arguments: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRun {
    pub sub_id: SubmissionId,
    pub actor_id: ActorID,
    pub task_description: Option<String>,
    pub messages: Vec<ChatLogEntry>,
    pub tools: Vec<ToolEventEntry>,
    pub result: Option<TaskResult>,
}

impl ChatRun {
    pub fn to_index_text(&self) -> String {
        let mut sections = Vec::new();
        sections.push(format!("Task: {}", self.task_description.clone().unwrap_or_default()));
        if !self.messages.is_empty() {
            sections.push("Conversation:".to_string());
            for m in &self.messages {
                sections.push(format!("- {} [{}]: {}", m.role, m.message_type, m.content));
            }
        }
        if !self.tools.is_empty() {
            sections.push("Tools:".to_string());
            for t in &self.tools {
                let res = if t.success {
                    format!("ok {:?}", t.result.as_ref())
                } else {
                    format!("error {:?}", t.error.as_ref())
                };
                sections.push(format!("- {} ({}) -> {}", t.tool_name, t.arguments, res));
            }
        }
        if let Some(TaskResult::Value(v)) = &self.result {
            sections.push(format!("Outcome: {}", v));
        }
        sections.join("\n")
    }
}

#[derive(Debug, Default, Clone)]
pub struct RagAnalysisCenter {
    pub runs: Arc<RwLock<HashMap<SubmissionId, ChatRun>>>,
    pub engine: Arc<RagEngine>,
}

impl RagAnalysisCenter {
    pub fn new(engine: Arc<RagEngine>) -> Self {
        Self { runs: Arc::new(RwLock::new(HashMap::new())), engine }
    }

    pub fn ingest(&self, event: &Event) {
        use Event::*;
        match event {
            TaskStarted { sub_id, actor_id, task_description } => {
                let mut guard = self.runs.write();
                guard.insert(*sub_id, ChatRun {
                    sub_id: *sub_id,
                    actor_id: *actor_id,
                    task_description: Some(task_description.clone()),
                    messages: Vec::new(),
                    tools: Vec::new(),
                    result: None,
                });
            }
            ChatMessageAppended { sub_id, actor_id, role, message_type, content } => {
                let mut guard = self.runs.write();
                let entry = guard.entry(*sub_id).or_insert(ChatRun {
                    sub_id: *sub_id,
                    actor_id: *actor_id,
                    task_description: None,
                    messages: Vec::new(),
                    tools: Vec::new(),
                    result: None,
                });
                entry.messages.push(ChatLogEntry { role: role.clone(), message_type: message_type.clone(), content: content.clone() });
            }
            ToolCallRequested { id, tool_name, arguments } => {
                let mut guard = self.runs.write();
                // We don't know sub_id here; rely on later completion to attach? We'll store a floating tool entry
                // For simplicity, do nothing here; completion will add full record
                let _ = (id, tool_name, arguments);
            }
            ToolCallCompleted { id, tool_name, result } => {
                let mut guard = self.runs.write();
                // Without sub_id, we can't attach to a specific run from this event alone; this is a limitation.
                // We'll add to all active runs' tools as a last-resort association.
                for (_k, run) in guard.iter_mut() {
                    run.tools.push(ToolEventEntry {
                        id: id.clone(),
                        tool_name: tool_name.clone(),
                        arguments: String::new(),
                        success: true,
                        result: Some(result.clone()),
                        error: None,
                    });
                }
            }
            ToolCallFailed { id, tool_name, error } => {
                let mut guard = self.runs.write();
                for (_k, run) in guard.iter_mut() {
                    run.tools.push(ToolEventEntry {
                        id: id.clone(),
                        tool_name: tool_name.clone(),
                        arguments: String::new(),
                        success: false,
                        result: None,
                        error: Some(error.clone()),
                    });
                }
            }
            TaskComplete { sub_id, result } => {
                let mut guard = self.runs.write();
                if let Some(run) = guard.get_mut(sub_id) {
                    run.result = Some(result.clone());
                }
            }
            _ => {}
        }
    }

    pub fn index_all(&self) {
        let runs = self.runs.read().clone();
        for (sub_id, run) in runs.into_iter() {
            let text = run.to_index_text();
            let _ = self.engine.store.upsert("analysis", vec![crate::VectorDoc {
                id: sub_id.to_string(),
                text,
                embedding: Vec::new(), // will be set by rag_index tool, but here we only upsert raw; clients should use rag_index tool normally
                metadata: Some(json!({"actor_id": run.actor_id.to_string()})),
            }]);
        }
    }
}

pub fn spawn_listener(mut receiver: ReceiverStream<Event>, center: Arc<RagAnalysisCenter>) -> JoinHandle<()> {
    tokio::spawn(async move {
        use tokio_stream::StreamExt;
        while let Some(event) = receiver.next().await {
            center.ingest(&event);
        }
    })
}

