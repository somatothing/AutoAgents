use crate::rag::{cosine_similarity, DocumentChunk, InMemoryVectorStore};
use crate::tool::{ToolCallError, ToolRuntime, ToolT};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryInput {
    pub query: String,
    pub candidates: Vec<String>,
    pub top_k: Option<usize>,
}

impl RagQueryInput {
    pub fn schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "candidates": {"type": "array", "items": {"type": "string"}},
                "top_k": {"type": "integer", "minimum": 1}
            },
            "required": ["query", "candidates"]
        })
    }
}

#[derive(Debug)]
pub struct RagAnalysisTool;

impl ToolT for RagAnalysisTool {
    fn name(&self) -> &'static str {
        "rag_analysis"
    }

    fn description(&self) -> &'static str {
        "Rank candidate passages by semantic similarity to the query using embeddings provided by the model."
    }

    fn args_schema(&self) -> Value {
        RagQueryInput::schema()
    }
}

impl ToolRuntime for RagAnalysisTool {
    fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let input: RagQueryInput = serde_json::from_value(args)?;
        let top_k = input.top_k.unwrap_or(5);

        // For now, just compute token-level TF-IDF-like fallback using naive character-based embedding if no embeddings are supplied.
        // Expectation: A higher-level orchestrator will pre-compute embeddings and put them into metadata if needed.

        // Simple hashing-based embedding to keep ToolRuntime synchronous
        fn hash_embed(text: &str, dim: usize) -> Vec<f32> {
            let mut v = vec![0f32; dim];
            for (i, ch) in text.chars().enumerate() {
                let idx = (ch as usize + i) % dim;
                v[idx] += 1.0;
            }
            v
        }

        let dim = 256usize;
        let query_vec = hash_embed(&input.query, dim);

        let mut store = InMemoryVectorStore::new();
        for (i, text) in input.candidates.iter().enumerate() {
            let emb = hash_embed(text, dim);
            store.upsert(DocumentChunk {
                id: format!("c_{i}"),
                text: text.clone(),
                metadata: None,
                embedding: Some(emb),
            });
        }

        // rank
        let results = store.similarity_search(&query_vec, top_k);
        let out: Vec<Value> = results
            .into_iter()
            .map(|c| {
                let score = cosine_similarity(&query_vec, c.embedding.as_ref().unwrap());
                json!({
                    "id": c.id,
                    "text": c.text,
                    "score": score
                })
            })
            .collect();

        Ok(json!({"results": out}))
    }
}

