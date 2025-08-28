use crate::planning::PlanGraph;

pub struct Executor<B> {
    backend: B,
}

impl<B> Executor<B> {
    pub fn new(backend: B) -> Self { Self { backend } }
    pub fn backend(&self) -> &B { &self.backend }
}

impl<B> Executor<B> {
    pub async fn run(&self, _plan: &PlanGraph) -> anyhow::Result<()> {
        Ok(())
    }
}

