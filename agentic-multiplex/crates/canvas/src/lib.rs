use artifacts::{Artifact, ArtifactKind};

#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self { Self { width, height } }
    pub fn to_artifact(&self) -> Artifact {
        Artifact { id: "canvas".into(), kind: ArtifactKind::Canvas, content: vec![], mime: "application/octet-stream".into() }
    }
}
