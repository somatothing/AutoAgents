use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError { #[error("not implemented")] NotImplemented }

pub trait Embedder { fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>; }

pub struct NoopEmbedder;
impl Embedder for NoopEmbedder { fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> { Ok(vec![]) } }
