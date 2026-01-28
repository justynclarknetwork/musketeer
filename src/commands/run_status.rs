use std::env;
use std::fs;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::{layout, read};
use crate::model::progress::ProgressLog;
use crate::model::run::Plan;

pub fn run(replay: Option<String>) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let runs_dir = layout::runs_dir(&root);
    if !runs_dir.exists() {
        return Err(MusketeerError::WorkspaceMissing(runs_dir.display().to_string()).into());
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
        let summary = summarize_run(&root, &replay_id)?;
        println!("{summary}");
        return Ok(());
    }

    if entries.is_empty() {
        println!("no runs found");
        return Ok(());
    }

    for replay_id in entries {
        let summary = summarize_run(&root, &replay_id)?;
        println!("{summary}");
    }

    Ok(())
}

fn summarize_run(root: &std::path::Path, replay_id: &str) -> anyhow::Result<String> {
    let plan: Plan = read::read_yaml(&layout::plan_path(root, replay_id))?;
    let progress: ProgressLog = read::read_yaml(&layout::progress_path(root, replay_id))?;

    let total_tasks = plan.tasks.len();
    let done_tasks = plan
        .tasks
        .iter()
        .filter(|task| task.status == "done")
        .count();
    let last_ts = progress.entries.last().map(|entry| entry.ts.as_str());
    let last_ts_display = last_ts.unwrap_or("-");

    Ok(format!(
        "{replay_id} tasks {done}/{total} last {last}",
        replay_id = replay_id,
        done = done_tasks,
        total = total_tasks,
        last = last_ts_display
    ))
}
