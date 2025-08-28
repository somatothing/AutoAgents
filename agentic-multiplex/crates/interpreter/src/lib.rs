use artifacts::Artifact;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("execution error: {0}")]
    Execution(String),
}

pub trait Interpreter {
    fn execute(&self, language: &str, code: &str) -> Result<Artifact, InterpreterError>;
}

pub struct NoopInterpreter;
impl Interpreter for NoopInterpreter {
    fn execute(&self, language: &str, code: &str) -> Result<Artifact, InterpreterError> {
        Ok(Artifact { id: "noop".into(), kind: artifacts::ArtifactKind::Text, content: format!("{}", code).into_bytes(), mime: "text/plain".into() })
    }
}
