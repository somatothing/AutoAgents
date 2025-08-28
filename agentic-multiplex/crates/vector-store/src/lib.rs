use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedItem { pub id: String, pub vector: Vec<f32> }

#[derive(Default, Clone)]
pub struct InMemoryIndex { inner: Arc<Mutex<Vec<IndexedItem>>> }

impl InMemoryIndex {
    pub fn new() -> Self { Self::default() }
    pub fn upsert(&self, id: String, vector: Vec<f32>) {
        let mut guard = self.inner.lock().unwrap();
        guard.retain(|x| x.id != id);
        guard.push(IndexedItem { id, vector });
    }
    pub fn query_by_vector(&self, vector: &[f32], top_k: usize) -> Vec<IndexedItem> {
        let guard = self.inner.lock().unwrap();
        let mut scored: Vec<_> = guard.iter().cloned().map(|it| {
            let score = cosine_similarity(&it.vector, vector);
            (score, it)
        }).collect();
        scored.sort_by(|a,b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(_, it)| it).collect()
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x,y)| x*y).sum();
    let na = (a.iter().map(|x| x*x).sum::<f32>()).sqrt();
    let nb = (b.iter().map(|x| x*x).sum::<f32>()).sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

pub fn upsert_text<E: embeddings::Embedder>(index: &InMemoryIndex, id: String, text: &str, embedder: &E) -> Result<(), embeddings::EmbeddingError> {
    let vec = embedder.embed(text)?;
    index.upsert(id, vec);
    Ok(())
}

pub fn query_text<E: embeddings::Embedder>(index: &InMemoryIndex, text: &str, top_k: usize, embedder: &E) -> Result<Vec<IndexedItem>, embeddings::EmbeddingError> {
    let vec = embedder.embed(text)?;
    Ok(index.query_by_vector(&vec, top_k))
}
