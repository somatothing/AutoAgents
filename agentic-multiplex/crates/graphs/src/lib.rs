#[derive(Debug, Clone)]
pub struct Node { pub id: String }
#[derive(Debug, Clone)]
pub struct Edge { pub from: String, pub to: String }
#[derive(Debug, Clone, Default)]
pub struct Graph { pub nodes: Vec<Node>, pub edges: Vec<Edge> }
