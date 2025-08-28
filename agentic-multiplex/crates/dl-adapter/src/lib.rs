use agent_core::Think;

#[cfg(feature = "rotta")]
pub mod rotta_backend {
    use super::*;
    use rotta_rs as rotta;

    pub struct RottaThinker;
    impl agent_core::Think for RottaThinker {
        fn think(&self, input: &str) -> Vec<String> {
            // Placeholder: integrate with rotta_rs tensors/models here
            vec![format!("rotta step: {}", input)]
        }
    }
}

#[cfg(feature = "burn")]
pub mod burn_backend {
    use super::*;
    use burn_core as burn;

    pub struct BurnThinker;
    impl agent_core::Think for BurnThinker {
        fn think(&self, input: &str) -> Vec<String> {
            // Placeholder: integrate with burn_core tensors/models here
            vec![format!("burn step: {}", input)]
        }
    }
}

pub struct EchoThinker;
impl Think for EchoThinker {
    fn think(&self, input: &str) -> Vec<String> { vec![input.to_string()] }
}
