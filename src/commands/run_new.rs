use std::env;

use anyhow::Context;
use uuid::Uuid;

use crate::error::MusketeerError;
use crate::fs::{layout, write};
use crate::model::progress::ProgressLog;
use crate::model::run::{Constraints, Handoff, Intent, Plan};
use crate::musketeer_namespace;
use crate::output;
use crate::small_workspace;
use crate::workspace_mode::{self, WorkspaceContext};

pub fn run(json_mode: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let ctx = workspace_mode::resolve(&root)?;

    match &ctx {
        WorkspaceContext::SmallNative { replay_id, .. } => {
            // In SMALL-native mode, intent/constraints/plan/progress/handoff
            // belong to SMALL. Musketeer only creates its execution structure.
            let missing = small_workspace::missing_artifacts(&root);
            if !missing.is_empty() {
                return Err(MusketeerError::HandoffInvalid(format!(
                    "SMALL workspace incomplete, missing: {}. Run `small init` first.",
                    missing.join(", ")
                ))
                .into());
            }

            let rid = replay_id.as_deref().unwrap_or("default");
            let rid = if rid == "default" {
                Uuid::new_v4().to_string()
            } else {
                rid.to_string()
            };

            // Create .musketeer/runs/<replayId>/ structure only
            musketeer_namespace::ensure_runs_dir(&root)?;
            let run_dir = layout::state_dir(&root).join("runs").join(&rid);
            write::ensure_dir(&run_dir)?;

            if json_mode {
                output::emit_ok(
                    json_mode,
                    Some(&rid),
                    serde_json::json!({"replay_id": rid, "mode": "small_native"}),
                );
            } else {
                println!("prepared SMALL-native run {rid}");
            }
        }
        WorkspaceContext::Legacy { .. } => {
            let state_dir = layout::state_dir(&root);
            if !state_dir.exists() {
                return Err(
                    MusketeerError::WorkspaceMissing(state_dir.display().to_string()).into(),
                );
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
                verdict: None,
                verdict_reason: None,
            };

            eprintln!("[deprecated] Creating legacy Musketeer artifacts. Future versions will require a SMALL workspace (.small/).");

            write::write_yaml(&layout::intent_path(&root, &replay_id), &intent)?;
            write::write_yaml(&layout::constraints_path(&root, &replay_id), &constraints)?;
            write::write_yaml(&layout::plan_path(&root, &replay_id), &plan)?;
            write::write_yaml(&layout::progress_path(&root, &replay_id), &progress)?;
            write::write_yaml(&layout::handoff_path(&root, &replay_id), &handoff)?;

            if json_mode {
                output::emit_ok(
                    json_mode,
                    Some(&replay_id),
                    serde_json::json!({"replay_id": replay_id}),
                );
            } else {
                println!("prepared handoff record {replay_id}");
            }
        }
    }

    Ok(())
}
