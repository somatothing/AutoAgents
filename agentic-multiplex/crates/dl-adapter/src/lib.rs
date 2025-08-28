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

pub struct MultiplexThinker {
    #[cfg(feature = "rotta")] rotta: Option<rotta_backend::RottaThinker>,
    #[cfg(not(feature = "rotta"))] _r: (),
    #[cfg(feature = "burn")] burn: Option<burn_backend::BurnThinker>,
    #[cfg(not(feature = "burn"))] _b: (),
}

impl MultiplexThinker {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rotta")] rotta: Some(rotta_backend::RottaThinker),
            #[cfg(not(feature = "rotta"))] _r: (),
            #[cfg(feature = "burn")] burn: Some(burn_backend::BurnThinker),
            #[cfg(not(feature = "burn"))] _b: (),
        }
    }
}

impl Think for MultiplexThinker {
    fn think(&self, input: &str) -> Vec<String> {
        let mut steps: Vec<String> = Vec::new();
        #[cfg(feature = "rotta")]
        if let Some(t) = &self.rotta { steps.extend(t.think(input)); }
        #[cfg(feature = "burn")]
        if let Some(t) = &self.burn { steps.extend(t.think(input)); }
        if steps.is_empty() { steps.push(input.to_string()); }
        steps
    }
}
