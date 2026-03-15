use serde::{Deserialize, Serialize};

/// Musketeer verdict -- written to `.musketeer/verdicts/<replayId>.verdict.yml`.
///
/// This is Musketeer-owned state, separate from the SMALL handoff artifact.
#[derive(Debug, Serialize, Deserialize)]
pub struct Verdict {
    pub kind: String,
    pub musketeer_version: String,
    pub replay_id: String,
    pub status: String,
    pub gate: String,
    pub reason: String,
    pub timestamp: String,
}
