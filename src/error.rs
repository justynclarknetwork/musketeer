use thiserror::Error;

#[derive(Error, Debug)]
pub enum MusketeerError {
    #[error("workspace not initialized: missing {0}")]
    WorkspaceMissing(String),

    #[error("run not found: {0}")]
    RunNotFound(String),

    #[error("invariant failed: {0}")]
    InvariantFailed(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),
}
