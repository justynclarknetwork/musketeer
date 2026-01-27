use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub workspace: Workspace,
    pub agents: Agents,
    pub policy: Policy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub state_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agents {
    pub originator: AgentSeat,
    pub cross_examiner: AgentSeat,
    pub executor: AgentSeat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSeat {
    pub adapter: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    pub executor_allowlist: Vec<String>,
    pub redaction: Redaction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Redaction {
    pub enabled: bool,
    pub patterns: Vec<String>,
}
