use axum::{routing::{get, post}, Router, extract::State, Json};
use std::net::SocketAddr;
use agent_core::{Agent, AgentRequest, AgentResponse, MultiChunkMemory};
use dl_adapter::EchoThinker;
use qwen_client::{QwenClient, ChatRequest, ChatResponse};
use embeddings::{Embedder, NoopEmbedder};
use vector_store::{InMemoryIndex, upsert_text, query_text};
use orchestrator::RoundRobinOrchestrator;
use tools::{Calculator, CalculatorInput, TensorflowRunner, TensorflowTask};

#[derive(Clone, Default)]
struct AppState {
    qwen_base_url: String,
    qwen_api_key: Option<String>,
    index: InMemoryIndex,
}

async fn run_handler(_state: State<AppState>, Json(req): Json<AgentRequest>) -> Json<AgentResponse> {
    let thinker = EchoThinker;
    let mut agent = Agent::new(thinker, MultiChunkMemory::new());
    Json(agent.run(req))
}

async fn chat_handler(State(state): State<AppState>, Json(mut req): Json<ChatRequest>) -> Json<ChatResponse> {
    if req.model.is_empty() { req.model = "qwen-coder".into(); }
    let client = QwenClient::new(state.qwen_base_url.clone(), state.qwen_api_key.clone());
    let resp = client.chat(&req).await.unwrap_or(ChatResponse { content: "".into() });
    Json(resp)
}

#[derive(serde::Deserialize)]
struct EmbedInput { text: String }
#[derive(serde::Serialize)]
struct EmbedOutput { vector: Vec<f32> }

async fn embed_handler(State(state): State<AppState>, Json(input): Json<EmbedInput>) -> Json<EmbedOutput> {
    let e = NoopEmbedder;
    let v = e.embed(&input.text).unwrap_or_default();
    let _ = upsert_text(&state.index, format!("doc:{}", uuid::Uuid::new_v4()), &input.text, &e);
    Json(EmbedOutput { vector: v })
}

async fn calc_handler(_state: State<AppState>, Json(input): Json<CalculatorInput>) -> Json<String> {
    let calc = Calculator;
    let out = calc.run(&input).unwrap();
    Json(String::from_utf8_lossy(&out.content).to_string())
}

async fn tf_handler(_state: State<AppState>, bytes: axum::body::Bytes) -> String {
    let tf = TensorflowRunner;
    let art = tf.run(&TensorflowTask { graph_def: bytes.to_vec() });
    format!("tf artifact {} bytes", art.content.len())
}

#[derive(serde::Deserialize)]
struct RagQuery { text: String, top_k: Option<usize> }

async fn rag_handler(State(state): State<AppState>, Json(q): Json<RagQuery>) -> Json<Vec<String>> {
    let e = NoopEmbedder;
    let hits = query_text(&state.index, &q.text, q.top_k.unwrap_or(5), &e).unwrap_or_default();
    Json(hits.into_iter().map(|h| h.id).collect())
}

#[tokio::main]
async fn main() {
    let state = AppState {
        qwen_base_url: std::env::var("QWEN_BASE").unwrap_or_else(|_| "https://api.example.com".into()),
        qwen_api_key: std::env::var("QWEN_KEY").ok(),
        index: InMemoryIndex::new(),
    };
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/run", post(run_handler))
        .route("/chat", post(chat_handler))
        .route("/embed", post(embed_handler))
        .route("/calc", post(calc_handler))
        .route("/tf", post(tf_handler))
        .route("/rag", post(rag_handler))
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
