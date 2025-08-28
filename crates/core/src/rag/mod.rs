use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub text: String,
    pub metadata: Option<serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryVectorStore {
    pub dim: Option<usize>,
    pub data: HashMap<String, DocumentChunk>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, chunk: DocumentChunk) {
        if let Some(ref emb) = chunk.embedding {
            if self.dim.is_none() {
                self.dim = Some(emb.len());
            }
        }
        self.data.insert(chunk.id.clone(), chunk);
    }

    pub fn similarity_search(&self, query: &[f32], top_k: usize) -> Vec<&DocumentChunk> {
        let mut scored: Vec<(&DocumentChunk, f32)> = self
            .data
            .values()
            .filter_map(|c| c.embedding.as_ref().map(|e| (c, cosine_similarity(query, e))))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .map(|(c, _)| c)
            .collect()
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-8);
    dot / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_basic() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_in_memory_store() {
        let mut store = InMemoryVectorStore::new();
        store.upsert(DocumentChunk {
            id: "1".into(),
            text: "hello".into(),
            metadata: None,
            embedding: Some(vec![1.0, 0.0]),
        });
        store.upsert(DocumentChunk {
            id: "2".into(),
            text: "world".into(),
            metadata: None,
            embedding: Some(vec![0.0, 1.0]),
        });

        let results = store.similarity_search(&[1.0, 0.0], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
    }
}

