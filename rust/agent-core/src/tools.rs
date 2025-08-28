use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub json_schema: serde_json::Value,
}

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Vec<ToolSpec>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self { tools: Vec::new() }
    }
}

impl ToolRegistry {
    pub fn register(&mut self, spec: ToolSpec) { self.tools.push(spec); }
    pub fn list(&self) -> &[ToolSpec] { &self.tools }
}

