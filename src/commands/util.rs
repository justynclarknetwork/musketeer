use std::fs;
use std::path::Path;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::layout;

pub fn latest_replay_id(root: &Path) -> anyhow::Result<String> {
    let runs_dir = layout::runs_dir(root);
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
    entries
        .pop()
        .ok_or_else(|| MusketeerError::RunNotFound("no runs found".to_string()).into())
}
