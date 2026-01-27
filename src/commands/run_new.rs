use std::env;

use anyhow::Context;
use uuid::Uuid;

use crate::error::MusketeerError;
use crate::fs::{layout, write};
use crate::model::progress::ProgressLog;
use crate::model::run::{Constraints, Handoff, Intent, Plan};

pub fn run() -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let state_dir = layout::state_dir(&root);
    if !state_dir.exists() {
        return Err(MusketeerError::WorkspaceMissing(state_dir.display().to_string()).into());
    }

    let replay_id = Uuid::new_v4().to_string();
    let run_dir = layout::run_dir(&root, &replay_id);
    write::ensure_dir(&run_dir)?;

    let intent = Intent {
        replay_id: replay_id.clone(),
        title: "Untitled".to_string(),
        outcome: "TBD".to_string(),
    };
    let constraints = Constraints {
        replay_id: replay_id.clone(),
        scope: Vec::new(),
        non_goals: Vec::new(),
        allowlist: Vec::new(),
    };
    let plan = Plan {
        replay_id: replay_id.clone(),
        tasks: Vec::new(),
    };
    let progress = ProgressLog {
        replay_id: replay_id.clone(),
        entries: Vec::new(),
    };
    let handoff = Handoff {
        replay_id: replay_id.clone(),
        note: "".to_string(),
    };

    write::write_yaml(&layout::intent_path(&root, &replay_id), &intent)?;
    write::write_yaml(&layout::constraints_path(&root, &replay_id), &constraints)?;
    write::write_yaml(&layout::plan_path(&root, &replay_id), &plan)?;
    write::write_yaml(&layout::progress_path(&root, &replay_id), &progress)?;
    write::write_yaml(&layout::handoff_path(&root, &replay_id), &handoff)?;

    println!("created run {replay_id}");
    Ok(())
}
