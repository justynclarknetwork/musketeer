use std::collections::HashSet;
use std::path::Path;

use crate::fs::{layout, read};
use crate::model::progress::ProgressLog;
use crate::model::run::{Constraints, Handoff, Intent, Plan};

pub struct CheckResult {
    pub ok: bool,
    pub errors: Vec<String>,
}

pub fn check_run(workspace_root: &Path, replay_id: &str) -> CheckResult {
    let mut errors = Vec::new();
    let run_dir = layout::run_dir(workspace_root, replay_id);
    if !run_dir.exists() {
        errors.push(format!("handoff not found: {replay_id}"));
        return CheckResult { ok: false, errors };
    }

    let required_files = [
        ("intent", layout::intent_path(workspace_root, replay_id)),
        (
            "constraints",
            layout::constraints_path(workspace_root, replay_id),
        ),
        ("plan", layout::plan_path(workspace_root, replay_id)),
        ("progress", layout::progress_path(workspace_root, replay_id)),
        ("handoff", layout::handoff_path(workspace_root, replay_id)),
    ];

    for (label, path) in &required_files {
        if !path.exists() {
            errors.push(format!("missing required file {label}: {}", path.display()));
        }
    }

    let intent = read_yaml_with_errors::<Intent>(
        layout::intent_path(workspace_root, replay_id),
        "intent",
        &mut errors,
    );
    let constraints = read_yaml_with_errors::<Constraints>(
        layout::constraints_path(workspace_root, replay_id),
        "constraints",
        &mut errors,
    );
    let plan = read_yaml_with_errors::<Plan>(
        layout::plan_path(workspace_root, replay_id),
        "plan",
        &mut errors,
    );
    let progress = read_yaml_with_errors::<ProgressLog>(
        layout::progress_path(workspace_root, replay_id),
        "progress",
        &mut errors,
    );
    let handoff = read_yaml_with_errors::<Handoff>(
        layout::handoff_path(workspace_root, replay_id),
        "handoff",
        &mut errors,
    );

    let mut replay_ids = Vec::new();
    if let Some(intent) = intent.as_ref() {
        replay_ids.push(("intent", intent.replay_id.as_str()));
    }
    if let Some(constraints) = constraints.as_ref() {
        replay_ids.push(("constraints", constraints.replay_id.as_str()));
    }
    if let Some(plan) = plan.as_ref() {
        replay_ids.push(("plan", plan.replay_id.as_str()));
    }
    if let Some(progress) = progress.as_ref() {
        replay_ids.push(("progress", progress.replay_id.as_str()));
    }
    if let Some(handoff) = handoff.as_ref() {
        replay_ids.push(("handoff", handoff.replay_id.as_str()));
    }

    for (label, id) in &replay_ids {
        if *id != replay_id {
            errors.push(format!(
                "replay_id mismatch in {label}: expected {replay_id}, got {id}"
            ));
        }
    }

    if let Some(progress) = progress.as_ref() {
        let mut prev = 0;
        for entry in &progress.entries {
            if prev == 0 {
                if entry.seq != 1 {
                    errors.push("progress seq must start at 1".to_string());
                    break;
                }
            } else if entry.seq <= prev {
                errors.push("progress seq must be strictly increasing".to_string());
                break;
            }
            prev = entry.seq;
        }
    }

    if let Some(plan) = plan.as_ref() {
        let mut seen = HashSet::new();
        for task in &plan.tasks {
            if !seen.insert(task.id.clone()) {
                errors.push(format!("duplicate plan task id: {}", task.id));
                break;
            }
        }
    }

    CheckResult {
        ok: errors.is_empty(),
        errors,
    }
}

fn read_yaml_with_errors<T: serde::de::DeserializeOwned>(
    path: std::path::PathBuf,
    label: &str,
    errors: &mut Vec<String>,
) -> Option<T> {
    match read::read_yaml(&path) {
        Ok(value) => Some(value),
        Err(err) => {
            if path.exists() {
                errors.push(format!("failed to parse {label}: {err}"));
            }
            None
        }
    }
}
