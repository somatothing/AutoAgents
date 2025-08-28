use autoagents_core::core::tool::{ToolCallError, ToolInputT, ToolRuntime, ToolT};
use autoagents_core::core::tool;
use autoagents_core::core::tool::ToolCallResult;
use autoagents_llm::embedding::EmbeddingProvider;
use async_trait::async_trait;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct VectorDoc {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryVectorStore {
    pub namespace_to_docs: Arc<RwLock<HashMap<String, Vec<VectorDoc>>>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&self, namespace: &str, docs: Vec<VectorDoc>) {
        let mut guard = self.namespace_to_docs.write();
        let entry = guard.entry(namespace.to_string()).or_default();
        for doc in docs {
            if let Some(existing) = entry.iter_mut().find(|d| d.id == doc.id) {
                *existing = doc;
            } else {
                entry.push(doc);
            }
        }
    }

    pub fn query(&self, namespace: &str, query_embedding: &[f32], top_k: usize) -> Vec<(VectorDoc, f32)> {
        let guard = self.namespace_to_docs.read();
        let docs = guard.get(namespace).cloned().unwrap_or_default();
        let mut scored: Vec<(VectorDoc, f32)> = docs
            .into_iter()
            .map(|d| {
                let score = cosine_similarity(&d.embedding, query_embedding);
                (d, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        scored.into_iter().take(top_k).collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() { return 0.0; }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}

#[derive(Debug, Clone)]
pub struct RagEngine {
    pub store: InMemoryVectorStore,
}

impl RagEngine {
    pub fn new() -> Self { Self { store: InMemoryVectorStore::new() } }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RagIndexInput {
    pub namespace: String,
    pub documents: Vec<String>,
    pub metadatas: Option<Vec<Value>>, // optional same length
}

impl ToolInputT for RagIndexInput {
    fn io_schema() -> &'static str {
        r#"{"type":"object","properties":{"namespace":{"type":"string"},"documents":{"type":"array","items":{"type":"string"}},"metadatas":{"type":"array","items":{"type":"object"}}},"required":["namespace","documents"]}"#
    }
}

#[derive(Debug, Clone)]
pub struct RagIndexTool {
    pub engine: Arc<RagEngine>,
    pub embedder: Arc<dyn EmbeddingProvider>,
}

impl RagIndexTool { pub fn new(engine: Arc<RagEngine>, embedder: Arc<dyn EmbeddingProvider>) -> Self { Self { engine, embedder } } }

impl ToolT for RagIndexTool {
    fn name(&self) -> &'static str { "rag_index" }
    fn description(&self) -> &'static str { "Index documents into an in-memory vector store for retrieval-augmented generation." }
    fn args_schema(&self) -> Value { serde_json::from_str(<RagIndexInput as ToolInputT>::io_schema()).unwrap() }
}

impl ToolRuntime for RagIndexTool {
    fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let typed: RagIndexInput = serde_json::from_value(args)?;
        let texts = typed.documents.clone();
        let embeddings = futures::executor::block_on(self.embedder.embed(texts))
            .map_err(|e| ToolCallError::RuntimeError(e.to_string().into()))?;
        let docs: Vec<VectorDoc> = embeddings
            .into_iter()
            .enumerate()
            .map(|(i, emb)| VectorDoc {
                id: uuid::Uuid::new_v4().to_string(),
                text: typed.documents[i].clone(),
                embedding: emb,
                metadata: typed.metadatas.as_ref().and_then(|m| m.get(i).cloned()),
            })
            .collect();
        self.engine.store.upsert(&typed.namespace, docs);
        Ok(json!({"ok": true}))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RagQueryInput {
    pub namespace: String,
    pub query: String,
    pub top_k: Option<usize>,
}

impl ToolInputT for RagQueryInput {
    fn io_schema() -> &'static str {
        r#"{"type":"object","properties":{"namespace":{"type":"string"},"query":{"type":"string"},"top_k":{"type":"integer","minimum":1}},"required":["namespace","query"]}"#
    }
}

#[derive(Debug, Clone)]
pub struct RagQueryTool {
    pub engine: Arc<RagEngine>,
    pub embedder: Arc<dyn EmbeddingProvider>,
}

impl RagQueryTool { pub fn new(engine: Arc<RagEngine>, embedder: Arc<dyn EmbeddingProvider>) -> Self { Self { engine, embedder } } }

impl ToolT for RagQueryTool {
    fn name(&self) -> &'static str { "rag_query" }
    fn description(&self) -> &'static str { "Query relevant documents from the vector store and return top matches." }
    fn args_schema(&self) -> Value { serde_json::from_str(<RagQueryInput as ToolInputT>::io_schema()).unwrap() }
}

impl ToolRuntime for RagQueryTool {
    fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let typed: RagQueryInput = serde_json::from_value(args)?;
        let embedding = futures::executor::block_on(self.embedder.embed(vec![typed.query.clone()]))
            .map_err(|e| ToolCallError::RuntimeError(e.to_string().into()))?;
        let query_vec = embedding.into_iter().next().unwrap_or_default();
        let top_k = typed.top_k.unwrap_or(5);
        let results = self.engine.store.query(&typed.namespace, &query_vec, top_k);
        let serialized = results.into_iter().map(|(d, score)| json!({
            "id": d.id,
            "text": d.text,
            "score": score,
            "metadata": d.metadata
        })).collect_vec();
        Ok(json!({"matches": serialized}))
    }
}

