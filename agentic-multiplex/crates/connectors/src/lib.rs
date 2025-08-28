use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectorError { #[error("not implemented")] NotImplemented }

pub trait DatasetConnector { fn ingest(&self) -> Result<(), ConnectorError>; }

pub struct NoopConnector;
impl DatasetConnector for NoopConnector { fn ingest(&self) -> Result<(), ConnectorError> { Ok(()) } }
