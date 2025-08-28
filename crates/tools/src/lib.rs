use serde::{Deserialize, Serialize};
use thiserror::Error;
use artifacts::Artifact;
use interpreter::{Interpreter, InterpreterError, NoopInterpreter};

#[derive(Debug, Error)]
pub enum ToolError {
    #[error(transparent)] Interpreter(#[from] InterpreterError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorInput { pub expression: String }

pub struct Calculator;
impl Calculator { pub fn run(&self, input: &CalculatorInput) -> Result<Artifact, ToolError> {
    // For safety, pass to interpreter placeholder, later swap with safe-eval
    let interp = NoopInterpreter;
    Ok(interp.execute("text", &input.expression)?)
} }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorflowTask { pub graph_def: Vec<u8> }

pub struct TensorflowRunner;
impl TensorflowRunner { pub fn run(&self, task: &TensorflowTask) -> Artifact {
    Artifact { id: "tf".into(), kind: artifacts::ArtifactKind::Graph, content: task.graph_def.clone(), mime: "application/octet-stream".into() }
} }
