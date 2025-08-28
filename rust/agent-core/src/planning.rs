use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNode {
    pub id: Uuid,
    pub kind: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanGraph {
    pub nodes: Vec<PlanNode>,
    pub edges: Vec<PlanEdge>,
}

impl PlanGraph {
    pub fn add_step(&mut self, kind: impl Into<String>, prompt: impl Into<String>) -> Uuid {
        let id = Uuid::new_v4();
        self.nodes.push(PlanNode { id, kind: kind.into(), prompt: prompt.into() });
        id
    }

    pub fn connect(&mut self, from: Uuid, to: Uuid, label: impl Into<String>) {
        self.edges.push(PlanEdge { from, to, label: label.into() });
    }
}

