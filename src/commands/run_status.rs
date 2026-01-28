use std::env;
use std::fs;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::{layout, read};
use crate::model::progress::ProgressLog;
use crate::model::run::Plan;
use crate::ui::{interactive, mode, plain, pretty, RunSummary};

pub fn run(replay: Option<String>) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let runs_dir = layout::runs_dir(&root);
    if !runs_dir.exists() {
        return Err(MusketeerError::WorkspaceMissing(runs_dir.display().to_string()).into());
    }

    let entries = list_run_ids(&runs_dir)?;

    let tty = mode::CrosstermTty;
    let selected_mode = mode::select_mode(&tty);

    if let Some(replay_id) = replay {
        if !entries.contains(&replay_id) {
            return Err(MusketeerError::RunNotFound(replay_id).into());
        }
        let summary = summarize_run(&root, &replay_id)?;
        println!("{}", plain::status_line(&summary));
        return Ok(());
    }

    if entries.is_empty() {
        println!("no handoffs found");
        return Ok(());
    }

    let summaries = summarize_runs(&root, &entries)?;
    if selected_mode == mode::Mode::InteractiveEligible {
        let selected = interactive::run_status(|| {
            let entries = list_run_ids(&runs_dir)?;
            summarize_runs(&root, &entries)
        })?;
        if let Some(summary) = selected {
            println!("{}", plain::status_line(&summary));
        }
        return Ok(());
    }

    match selected_mode {
        mode::Mode::Plain => {
            for summary in &summaries {
                println!("{}", plain::status_line(summary));
            }
        }
        mode::Mode::Pretty | mode::Mode::InteractiveEligible => {
            println!("{}", pretty::status_table(&summaries));
        }
    }

    Ok(())
}

fn summarize_runs(root: &std::path::Path, entries: &[String]) -> anyhow::Result<Vec<RunSummary>> {
    entries
        .iter()
        .map(|replay_id| summarize_run(root, replay_id))
        .collect()
}

fn list_run_ids(runs_dir: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let mut entries: Vec<String> = fs::read_dir(runs_dir)
        .with_context(|| format!("failed to read {}", runs_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
        .collect();
    entries.sort();
    Ok(entries)
}

fn summarize_run(root: &std::path::Path, replay_id: &str) -> anyhow::Result<RunSummary> {
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

    Ok(RunSummary {
        replay_id: replay_id.to_string(),
        done: done_tasks,
        total: total_tasks,
        last: last_ts_display.to_string(),
    })
}
