use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub kind: String,
    pub uri: String,
    pub meta: serde_json::Value,
}

#[derive(Default)]
pub struct ArtifactStore {
    pub items: Vec<Artifact>,
}

impl ArtifactStore {
    pub fn add(&mut self, kind: impl Into<String>, uri: impl Into<String>, meta: serde_json::Value) -> Uuid {
        let id = Uuid::new_v4();
        self.items.push(Artifact { id, kind: kind.into(), uri: uri.into(), meta });
        id
    }
}

