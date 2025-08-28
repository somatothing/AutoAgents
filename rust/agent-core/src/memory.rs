use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    pub id: Uuid,
    pub label: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Default)]
pub struct MemoryStore {
    pub chunks: Vec<MemoryChunk>,
}

impl MemoryStore {
    pub fn add(&mut self, label: impl Into<String>, content: impl Into<String>, tags: Vec<String>) -> Uuid {
        let id = Uuid::new_v4();
        self.chunks.push(MemoryChunk { id, label: label.into(), content: content.into(), tags });
        id
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryChunk> {
        self.chunks.iter().filter(|c| c.tags.iter().any(|t| t == tag)).collect()
    }
}

