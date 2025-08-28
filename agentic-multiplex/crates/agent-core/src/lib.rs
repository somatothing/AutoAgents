use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub output: String,
}

pub trait Think {
    fn think(&self, input: &str) -> Vec<String>;
}

pub trait Memory {
    fn recall(&self, key: &str) -> Option<String>;
    fn store(&mut self, key: String, value: String);
}

pub struct MultiChunkMemory {
    chunks: std::collections::HashMap<String, String>,
}

impl MultiChunkMemory {
    pub fn new() -> Self { Self { chunks: Default::default() } }
}

impl Memory for MultiChunkMemory {
    fn recall(&self, key: &str) -> Option<String> { self.chunks.get(key).cloned() }
    fn store(&mut self, key: String, value: String) { self.chunks.insert(key, value); }
}

pub struct Agent<T: Think, M: Memory> {
    thinker: T,
    memory: M,
}

impl<T: Think, M: Memory> Agent<T, M> {
    pub fn new(thinker: T, memory: M) -> Self { Self { thinker, memory } }

    pub fn run(&mut self, req: AgentRequest) -> AgentResponse {
        let steps = self.thinker.think(&req.input);
        for (i, s) in steps.iter().enumerate() {
            self.memory.store(format!("step:{}", i), s.clone());
        }
        AgentResponse { output: steps.last().cloned().unwrap_or_default() }
    }
}
