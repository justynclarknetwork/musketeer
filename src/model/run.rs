use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Intent {
    pub replay_id: String,
    pub title: String,
    pub outcome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Constraints {
    pub replay_id: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub allowlist: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plan {
    pub replay_id: String,
    pub tasks: Vec<PlanTask>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanTask {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Handoff {
    pub replay_id: String,
    pub note: String,
}
