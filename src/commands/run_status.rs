use std::env;
use std::fs;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::{layout, read};
use crate::model::progress::ProgressLog;
use crate::model::run::Plan;
use crate::output;
use crate::small_adapter;
use crate::workspace_mode::{self, WorkspaceContext};

pub fn run(replay: Option<String>, json_mode: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let ctx = workspace_mode::resolve(&root)?;

    match &ctx {
        WorkspaceContext::SmallNative { .. } => {
            let rid = workspace_mode::resolve_replay_id(&ctx, replay.clone())?;

            let plan = small_adapter::read_plan(&root, &rid)?;
            let progress = small_adapter::read_progress(&root, &rid)?;

            let total = plan.tasks.len();
            let done = plan.tasks.iter().filter(|t| t.status == "done").count();
            let last_ts = progress
                .entries
                .last()
                .map(|e| e.ts.clone())
                .unwrap_or_else(|| "-".to_string());

            if json_mode {
                output::emit_ok(
                    json_mode,
                    Some(&rid),
                    serde_json::json!({
                        "mode": "small_native",
                        "done": done,
                        "total": total,
                        "last": last_ts
                    }),
                );
            } else {
                println!("[small] {rid} tasks {done}/{total} last {last_ts}");
            }
        }
        WorkspaceContext::Legacy { .. } => {
            workspace_mode::warn_legacy();

            let runs_dir = layout::runs_dir(&root);
            if !runs_dir.exists() {
                return Err(
                    MusketeerError::WorkspaceMissing(runs_dir.display().to_string()).into(),
                );
            }

            let mut entries: Vec<String> = fs::read_dir(&runs_dir)
                .with_context(|| format!("failed to read {}", runs_dir.display()))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_dir())
                .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                .collect();
            entries.sort();

            if let Some(replay_id) = replay {
                if !entries.contains(&replay_id) {
                    return Err(MusketeerError::RunNotFound(replay_id).into());
                }
                let (done, total, last_ts) = summarize_legacy_run(&root, &replay_id)?;
                if json_mode {
                    output::emit_ok(
                        json_mode,
                        Some(&replay_id),
                        serde_json::json!({"done": done, "total": total, "last": last_ts}),
                    );
                } else {
                    println!("{replay_id} tasks {done}/{total} last {last_ts}");
                }
                return Ok(());
            }

            if entries.is_empty() {
                if json_mode {
                    output::emit_ok(json_mode, None, serde_json::json!({"runs": []}));
                } else {
                    println!("no runs found");
                }
                return Ok(());
            }

            if json_mode {
                let mut runs = Vec::new();
                for replay_id in entries {
                    let (done, total, last_ts) = summarize_legacy_run(&root, &replay_id)?;
                    runs.push(serde_json::json!({"replay_id": replay_id, "done": done, "total": total, "last": last_ts}));
                }
                output::emit_ok(json_mode, None, serde_json::json!({"runs": runs}));
            } else {
                for replay_id in entries {
                    let (done, total, last_ts) = summarize_legacy_run(&root, &replay_id)?;
                    println!("{replay_id} tasks {done}/{total} last {last_ts}");
                }
            }
        }
    }

    Ok(())
}

fn summarize_legacy_run(
    root: &std::path::Path,
    replay_id: &str,
) -> anyhow::Result<(usize, usize, String)> {
    let plan: Plan = read::read_yaml(&layout::plan_path(root, replay_id))?;
    let progress: ProgressLog = read::read_yaml(&layout::progress_path(root, replay_id))?;

    let total_tasks = plan.tasks.len();
    let done_tasks = plan
        .tasks
        .iter()
        .filter(|task| task.status == "done")
        .count();
    let last_ts = progress
        .entries
        .last()
        .map(|entry| entry.ts.clone())
        .unwrap_or_else(|| "-".to_string());

    Ok((done_tasks, total_tasks, last_ts))
}
