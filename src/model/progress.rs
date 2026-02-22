use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressLog {
    pub replay_id: String,
    pub entries: Vec<ProgressEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub seq: u64,
    pub ts: String,
    pub role: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub summary: String,
}
