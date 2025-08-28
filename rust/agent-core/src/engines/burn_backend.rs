use super::{EngineBackend, InferenceRequest, InferenceResponse};
use async_trait::async_trait;

#[allow(unused_imports)]
use burn::tensor::backend::Backend as BurnBackend;

pub struct BurnEngine<B: BurnBackend> {
    _phantom: std::marker::PhantomData<B>,
}

impl<B: BurnBackend> BurnEngine<B> {
    pub fn new() -> Self {
        Self { _phantom: std::marker::PhantomData }
    }
}

#[async_trait]
impl<B: BurnBackend + Send + Sync> EngineBackend for BurnEngine<B> {
    async fn infer(&self, request: InferenceRequest) -> anyhow::Result<InferenceResponse> {
        let _ = request;
        Ok(InferenceResponse { output_text: String::from("[burn] stub output") })
    }
}

