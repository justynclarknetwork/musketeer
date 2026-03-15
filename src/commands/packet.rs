use std::env;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::{layout, read};
use crate::model::progress::ProgressLog;
use crate::model::run::{Constraints, Intent, Plan};
use crate::output;
use crate::small_adapter;
use crate::workspace_mode::{self, WorkspaceContext};

pub fn run(
    role: String,
    replay: Option<String>,
    _max_bytes: Option<usize>,
    json_mode: bool,
) -> anyhow::Result<()> {
    validate_role(&role)?;
    let root = env::current_dir().context("failed to resolve current dir")?;
    let ctx = workspace_mode::resolve(&root)?;
    let replay_id = workspace_mode::resolve_replay_id(&ctx, replay)?;

    let (intent, constraints, plan, progress) = match &ctx {
        WorkspaceContext::SmallNative { .. } => {
            let intent = small_adapter::read_intent(&root, &replay_id)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("intent: {}", e)))?;
            let constraints = small_adapter::read_constraints(&root, &replay_id)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("constraints: {}", e)))?;
            let plan = small_adapter::read_plan(&root, &replay_id)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("plan: {}", e)))?;
            let progress = small_adapter::read_progress(&root, &replay_id)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("progress: {}", e)))?;
            (intent, constraints, plan, progress)
        }
        WorkspaceContext::Legacy { .. } => {
            workspace_mode::warn_legacy();
            let intent: Intent = read::read_yaml(&layout::intent_path(&root, &replay_id))
                .map_err(|_| MusketeerError::HandoffInvalid("intent missing".to_string()))?;
            let constraints: Constraints =
                read::read_yaml(&layout::constraints_path(&root, &replay_id))
                    .map_err(|_| {
                        MusketeerError::HandoffInvalid("constraints missing".to_string())
                    })?;
            let plan: Plan = read::read_yaml(&layout::plan_path(&root, &replay_id))
                .map_err(|_| MusketeerError::HandoffInvalid("plan missing".to_string()))?;
            let progress: ProgressLog =
                read::read_yaml(&layout::progress_path(&root, &replay_id))
                    .map_err(|_| MusketeerError::HandoffInvalid("progress missing".to_string()))?;
            (intent, constraints, plan, progress)
        }
    };

    let plan_slice: Vec<_> = plan
        .tasks
        .iter()
        .filter(|task| task.status == "pending" || task.status == "in_progress")
        .take(10)
        .map(|task| serde_json::json!({"id": task.id, "title": task.title, "status": task.status}))
        .collect();

    let progress_slice: Vec<_> = progress
        .entries
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|e| {
            serde_json::json!({"seq": e.seq, "ts": e.ts, "role": e.role, "kind": e.kind, "message": if e.message.is_empty() { e.summary.clone() } else { e.message.clone() }})
        })
        .collect();

    let next_expected_action = if plan_slice.is_empty() {
        "review_or_close".to_string()
    } else {
        "execute_next_task".to_string()
    };

    let payload = serde_json::json!({
        "replay_id": replay_id,
        "role": role,
        "intent": {"title": intent.title, "outcome": intent.outcome},
        "constraints": constraints,
        "plan_slice": plan_slice,
        "progress_slice": progress_slice,
        "next_expected_action": next_expected_action
    });

    if json_mode {
        let rid = replay_id.clone();
        output::emit_ok(json_mode, Some(&rid), payload);
    } else {
        println!("packet ready for role");
    }

    Ok(())
}

fn validate_role(role: &str) -> anyhow::Result<()> {
    if role == "planner" || role == "executor" || role == "auditor" {
        Ok(())
    } else {
        Err(MusketeerError::RoleViolation(role.to_string()).into())
    }
}
