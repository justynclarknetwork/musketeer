use serde::{Deserialize, Serialize};

/// Musketeer-namespaced execution log.
///
/// This is Musketeer-owned state written to `.musketeer/runs/<replayId>/execution-log.yml`.
/// It replaces legacy progress writes in SMALL-native mode. It is NOT a shadow
/// of SMALL's canonical `progress.small.yml` -- it is Musketeer's own execution diary.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub kind: String,
    pub musketeer_version: String,
    pub replay_id: String,
    pub entries: Vec<ExecutionEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionEntry {
    pub seq: u64,
    pub ts: String,
    pub role: String,
    pub kind: String,
    pub message: String,
}

impl ExecutionLog {
    pub fn new(replay_id: &str) -> Self {
        Self {
            kind: "musketeer_execution_log".to_string(),
            musketeer_version: env!("CARGO_PKG_VERSION").to_string(),
            replay_id: replay_id.to_string(),
            entries: Vec::new(),
        }
    }
}
