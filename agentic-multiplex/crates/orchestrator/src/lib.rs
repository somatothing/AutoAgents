use agent_core::{Agent, AgentRequest, AgentResponse, Memory, Think};

pub struct RoundRobinOrchestrator<T: Think, M: Memory> { agents: Vec<Agent<T,M>> }

impl<T: Think, M: Memory> RoundRobinOrchestrator<T, M> {
    pub fn new(agents: Vec<Agent<T,M>>) -> Self { Self { agents } }
    pub fn run_all(&mut self, req: &AgentRequest) -> Vec<AgentResponse> {
        self.agents.iter_mut().map(|a| a.run(AgentRequest { input: req.input.clone() })).collect()
    }
}
