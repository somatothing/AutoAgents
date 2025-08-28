use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactKind {
    Text,
    Image,
    Graph,
    Canvas,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub content: Vec<u8>,
    pub mime: String,
}
