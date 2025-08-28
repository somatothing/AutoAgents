use axum::{routing::post, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;
use serde::{Deserialize, Serialize};

use agent_core::engines::qwen_openai::OpenAICompatibleEngine;
use agent_core::engines::{EngineBackend, InferenceRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any);

    let app = Router::new()
        .route("/api/chat", post(chat_handler))
        .layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8787));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
struct ChatInput {
    prompt: String,
}

#[derive(Serialize)]
struct ChatOutput {
    output: String,
}

async fn chat_handler(axum::Json(input): axum::Json<ChatInput>) -> axum::Json<ChatOutput> {
    let engine = match OpenAICompatibleEngine::from_env() {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(?err, "engine init failed");
            return axum::Json(ChatOutput { output: String::from("engine not configured") });
        }
    };
    let req = InferenceRequest { prompt: input.prompt, system: None, tools: vec![], max_tokens: Some(256) };
    let out = match engine.infer(req).await {
        Ok(r) => r.output_text,
        Err(err) => {
            tracing::error!(?err, "infer failed");
            String::from("error")
        }
    };
    axum::Json(ChatOutput { output: out })
}

