use std::sync::Arc;

use autoagents::core::agent::prebuilt::executor::{AutoGenExecutor, ReActExecutor};
use autoagents::core::agent::{AgentBuilder, AgentDeriveT};
use autoagents::core::tool::rag::RagAnalysisTool;
use autoagents::llm::builder::LLMBuilder;
use autoagents::llm::backends::openai::OpenAI;
use autoagents::llm::chat::StructuredOutputFormat;
use autoagents_derive::agent;

#[derive(Debug, Clone)]
struct DemoOutput { answer: String }

#[agent(
    name = "RAG AutoGen Demo",
    description = "Simple demo agent using AutoGen executor and RagAnalysis tool",
    output = DemoOutput,
    tools = [RagAnalysisTool]
)]
struct DemoAgent;

impl ReActExecutor for DemoAgent {}

#[tokio::main]
async fn main() {
    autoagents::init_logging();
    let llm: Arc<OpenAI> = LLMBuilder::<OpenAI>::new()
        .model(std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()))
        .build()
        .expect("LLM build failed");

    let agent = DemoAgent {};
    let handle = AgentBuilder::new(agent)
        .with_llm(llm)
        .build()
        .await
        .expect("Agent build failed");

    println!("Demo agent ready: {}", handle.agent.name());
}

