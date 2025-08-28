use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError { #[error("not implemented")] NotImplemented }

pub trait Embedder { fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>; }

pub struct NoopEmbedder;
impl Embedder for NoopEmbedder { fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> { Ok(vec![]) } }

pub struct HashingEmbedder { pub dim: usize }
impl Default for HashingEmbedder { fn default() -> Self { Self { dim: 256 } } }
impl Embedder for HashingEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut v = vec![0f32; self.dim];
        for (i, b) in text.as_bytes().iter().enumerate() {
            let idx = (i.wrapping_add(*b as usize)) % self.dim;
            v[idx] += 1.0;
        }
        let norm = (v.iter().map(|x| x * x).sum::<f32>()).sqrt();
        if norm > 0.0 { for x in &mut v { *x /= norm; } }
        Ok(v)
    }
}
