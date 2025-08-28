use agent_core::Think;

#[cfg(feature = "rotta")]
pub mod rotta_backend {
    use super::*;
    use rotta_rs as rotta;
    use rotta::Tensor as RTensor;

    pub struct RottaThinker;
    impl agent_core::Think for RottaThinker {
        fn think(&self, input: &str) -> Vec<String> {
            let t = RTensor::from_element(vec![2, 2], 1.0);
            vec![format!("rotta step: {} shape={:?}", input, t.value().shape)]
        }
    }
}

#[cfg(feature = "burn")]
pub mod burn_backend {
    use super::*;
    use burn_core as burn;
    use burn::prelude::*;

    pub struct BurnThinker;
    impl agent_core::Think for BurnThinker {
        fn think(&self, input: &str) -> Vec<String> {
            type B = burn_ndarray::NdArray<f32>;
            let device = <B as Backend>::Device::default();
            let a = Tensor::<B, 2>::zeros([2, 2], &device);
            let b = Tensor::<B, 2>::ones([2, 2], &device);
            let c = a + b;
            vec![format!("burn step: {} sum={}", input, c.sum().into_scalar())]
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
