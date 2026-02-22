use thiserror::Error;

#[derive(Error, Debug)]
pub enum MusketeerError {
    #[error("workspace not initialized: missing {0}")]
    WorkspaceMissing(String),

    #[error("handoff not found: {0}")]
    RunNotFound(String),

    #[error("invariant failed: {0}")]
    InvariantFailed(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("role violation: {0}")]
    RoleViolation(String),

    #[error("invalid or missing handoff material: {0}")]
    HandoffInvalid(String),

    #[error("auditor rejected: {0}")]
    VerdictRejected(String),
}

impl MusketeerError {
    pub fn exit_code(&self) -> i32 {
        match self {
            MusketeerError::InvariantFailed(_) => 20,
            MusketeerError::RoleViolation(_) => 21,
            MusketeerError::HandoffInvalid(_) => 22,
            MusketeerError::VerdictRejected(_) => 23,
            MusketeerError::WorkspaceMissing(_) | MusketeerError::RunNotFound(_) => 30,
            MusketeerError::InvalidInput(_) => 40,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            MusketeerError::InvariantFailed(_) => "E_INVARIANT_FAILED",
            MusketeerError::RoleViolation(_) => "E_ROLE_VIOLATION",
            MusketeerError::HandoffInvalid(_) => "E_HANDOFF_INVALID",
            MusketeerError::VerdictRejected(_) => "E_VERDICT_REJECTED",
            MusketeerError::WorkspaceMissing(_) | MusketeerError::RunNotFound(_) => {
                "E_WORKSPACE_INVALID"
            }
            MusketeerError::InvalidInput(_) => "E_INVALID_INPUT",
        }
    }
}
