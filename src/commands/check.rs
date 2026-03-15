use std::env;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::read;
use crate::invariants::check::check_run;
use crate::model::verdict::Verdict;
use crate::musketeer_namespace;
use crate::output;
use crate::small_adapter;
use crate::small_workspace;
use crate::workspace_mode::{self, WorkspaceContext};

pub fn run(replay: Option<String>, json_mode: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let ctx = workspace_mode::resolve(&root)?;
    let replay_id = workspace_mode::resolve_replay_id(&ctx, replay)?;

    match &ctx {
        WorkspaceContext::SmallNative { .. } => {
            // Phase 1: Validate SMALL state -- all canonical artifacts must exist and parse
            let missing = small_workspace::missing_artifacts(&root);
            if !missing.is_empty() {
                return Err(MusketeerError::HandoffInvalid(format!(
                    "SMALL workspace missing artifacts: {}",
                    missing.join(", ")
                ))
                .into());
            }

            // Validate each artifact parses correctly
            small_adapter::load_intent(&root)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("intent parse: {}", e)))?;
            small_adapter::load_constraints(&root)
                .map_err(|e| {
                    MusketeerError::HandoffInvalid(format!("constraints parse: {}", e))
                })?;
            small_adapter::load_plan(&root)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("plan parse: {}", e)))?;
            small_adapter::load_progress(&root)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("progress parse: {}", e)))?;
            small_adapter::load_handoff(&root)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("handoff parse: {}", e)))?;

            // Phase 2: Validate Musketeer execution state (verdict)
            let verdict_path = musketeer_namespace::verdict_path(&root, &replay_id);
            if verdict_path.is_file() {
                let v: Verdict = read::read_yaml(&verdict_path)?;
                if v.status == "reject" {
                    return Err(
                        MusketeerError::VerdictRejected("auditor rejected".to_string()).into(),
                    );
                }
            }
        }
        WorkspaceContext::Legacy { .. } => {
            workspace_mode::warn_legacy();

            let result = check_run(&root, &replay_id);
            if !result.ok {
                return Err(
                    MusketeerError::InvariantFailed(result.errors.join("; ")).into()
                );
            }

            // Check verdict (new location first, then legacy handoff)
            let verdict_path = musketeer_namespace::verdict_path(&root, &replay_id);
            let rejected = if verdict_path.is_file() {
                let v: Verdict = read::read_yaml(&verdict_path)?;
                v.status == "reject"
            } else {
                let handoff_path = crate::fs::layout::handoff_path(&root, &replay_id);
                if handoff_path.is_file() {
                    let handoff: crate::model::run::Handoff = read::read_yaml(&handoff_path)?;
                    handoff.verdict.as_deref() == Some("reject")
                } else {
                    false
                }
            };

            if rejected {
                return Err(
                    MusketeerError::VerdictRejected("auditor rejected".to_string()).into(),
                );
            }
        }
    }

    if json_mode {
        output::emit_ok(json_mode, Some(&replay_id), serde_json::json!({}));
    } else {
        println!("check ok: {replay_id}");
    }
    Ok(())
}
