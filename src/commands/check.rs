use std::env;
use std::fs;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::layout;
use crate::invariants::check::check_run;

pub fn run(replay: Option<String>) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let runs_dir = layout::runs_dir(&root);
    if !runs_dir.exists() {
        return Err(MusketeerError::WorkspaceMissing(runs_dir.display().to_string()).into());
    }

    let replay_id = match replay {
        Some(id) => id,
        None => latest_run_id(&runs_dir)?
            .ok_or_else(|| MusketeerError::RunNotFound("no runs found".to_string()))?,
    };

    let result = check_run(&root, &replay_id);
    if result.ok {
        println!("check ok: {replay_id}");
        return Ok(());
    }

    for error in result.errors {
        eprintln!("{error}");
    }
    Err(MusketeerError::InvariantFailed(replay_id).into())
}

fn latest_run_id(runs_dir: &std::path::Path) -> anyhow::Result<Option<String>> {
    let mut entries: Vec<String> = fs::read_dir(runs_dir)
        .with_context(|| format!("failed to read {}", runs_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
        .collect();
    entries.sort();
    Ok(entries.pop())
}
