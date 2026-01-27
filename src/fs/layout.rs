use std::path::{Path, PathBuf};

pub const STATE_DIR: &str = ".musketeer";
pub const RUNS_DIR: &str = "runs";

pub const CONFIG_FILE: &str = "musketeer.yml";

pub const INTENT_FILE: &str = "intent.yml";
pub const CONSTRAINTS_FILE: &str = "constraints.yml";
pub const PLAN_FILE: &str = "plan.yml";
pub const PROGRESS_FILE: &str = "progress.yml";
pub const HANDOFF_FILE: &str = "handoff.yml";

pub fn state_dir(root: &Path) -> PathBuf {
    root.join(STATE_DIR)
}

pub fn runs_dir(root: &Path) -> PathBuf {
    root.join(STATE_DIR).join(RUNS_DIR)
}

pub fn config_path(root: &Path) -> PathBuf {
    root.join(STATE_DIR).join(CONFIG_FILE)
}

pub fn run_dir(root: &Path, replay_id: &str) -> PathBuf {
    root.join(STATE_DIR).join(RUNS_DIR).join(replay_id)
}

pub fn intent_path(root: &Path, replay_id: &str) -> PathBuf {
    run_dir(root, replay_id).join(INTENT_FILE)
}

pub fn constraints_path(root: &Path, replay_id: &str) -> PathBuf {
    run_dir(root, replay_id).join(CONSTRAINTS_FILE)
}

pub fn plan_path(root: &Path, replay_id: &str) -> PathBuf {
    run_dir(root, replay_id).join(PLAN_FILE)
}

pub fn progress_path(root: &Path, replay_id: &str) -> PathBuf {
    run_dir(root, replay_id).join(PROGRESS_FILE)
}

pub fn handoff_path(root: &Path, replay_id: &str) -> PathBuf {
    run_dir(root, replay_id).join(HANDOFF_FILE)
}
