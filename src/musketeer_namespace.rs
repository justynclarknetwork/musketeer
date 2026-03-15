//! Musketeer namespace path helpers.
//!
//! Musketeer-owned state lives under `.musketeer/`. This module provides
//! helpers for the execution-layer subdirectories: packets, verdicts,
//! runs, and bridge.

use std::path::{Path, PathBuf};

use crate::fs::layout;

pub const PACKETS_DIR: &str = "packets";
pub const VERDICTS_DIR: &str = "verdicts";
pub const RUNS_DIR: &str = "runs";
pub const BRIDGE_DIR: &str = "bridge";

pub fn packets_dir(root: &Path) -> PathBuf {
    layout::state_dir(root).join(PACKETS_DIR)
}

pub fn verdicts_dir(root: &Path) -> PathBuf {
    layout::state_dir(root).join(VERDICTS_DIR)
}

pub fn runs_dir(root: &Path) -> PathBuf {
    layout::state_dir(root).join(RUNS_DIR)
}

pub fn bridge_dir(root: &Path) -> PathBuf {
    layout::state_dir(root).join(BRIDGE_DIR)
}

/// Path for a specific run directory: `.musketeer/runs/<replayId>/`
pub fn run_dir(root: &Path, replay_id: &str) -> PathBuf {
    runs_dir(root).join(replay_id)
}

/// Path for a verdict file: `.musketeer/verdicts/<replayId>.verdict.yml`
pub fn verdict_path(root: &Path, replay_id: &str) -> PathBuf {
    verdicts_dir(root).join(format!("{}.verdict.yml", replay_id))
}

/// Path for execution log: `.musketeer/runs/<replayId>/execution-log.yml`
pub fn execution_log_path(root: &Path, replay_id: &str) -> PathBuf {
    run_dir(root, replay_id).join("execution-log.yml")
}

/// Legacy artifact filenames that must never be written in SMALL-native mode.
const LEGACY_ARTIFACT_NAMES: &[&str] = &[
    "intent.yml",
    "constraints.yml",
    "plan.yml",
    "progress.yml",
    "handoff.yml",
];

/// Assert that a path is not a legacy artifact. Panics in SMALL-native mode
/// if the path ends with a legacy artifact filename under a runs directory.
///
/// Call this from write helpers or tests to enforce write-path purity.
pub fn assert_not_legacy_artifact(path: &Path, is_small_native: bool) -> anyhow::Result<()> {
    if !is_small_native {
        return Ok(());
    }
    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        if LEGACY_ARTIFACT_NAMES.contains(&file_name) {
            // Check if this is under a runs directory
            let path_str = path.to_string_lossy();
            if path_str.contains(".musketeer/runs/") || path_str.contains(".musketeer\\runs\\") {
                anyhow::bail!(
                    "SMALL-native write guard: refusing to write legacy artifact '{}' at {}",
                    file_name,
                    path.display()
                );
            }
        }
    }
    Ok(())
}

/// Ensure a namespace directory exists.
pub fn ensure_packets_dir(root: &Path) -> anyhow::Result<()> {
    crate::fs::write::ensure_dir(&packets_dir(root))
}

pub fn ensure_verdicts_dir(root: &Path) -> anyhow::Result<()> {
    crate::fs::write::ensure_dir(&verdicts_dir(root))
}

pub fn ensure_runs_dir(root: &Path) -> anyhow::Result<()> {
    crate::fs::write::ensure_dir(&runs_dir(root))
}

pub fn ensure_bridge_dir(root: &Path) -> anyhow::Result<()> {
    crate::fs::write::ensure_dir(&bridge_dir(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn path_helpers() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        assert_eq!(packets_dir(root), root.join(".musketeer").join("packets"));
        assert_eq!(verdicts_dir(root), root.join(".musketeer").join("verdicts"));
        assert_eq!(
            verdict_path(root, "abc-123"),
            root.join(".musketeer")
                .join("verdicts")
                .join("abc-123.verdict.yml")
        );
        assert_eq!(
            run_dir(root, "abc-123"),
            root.join(".musketeer").join("runs").join("abc-123")
        );
        assert_eq!(
            execution_log_path(root, "abc-123"),
            root.join(".musketeer")
                .join("runs")
                .join("abc-123")
                .join("execution-log.yml")
        );
    }

    #[test]
    fn guard_blocks_legacy_writes_in_small_mode() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let legacy_path = root.join(".musketeer/runs/test-id/progress.yml");
        assert!(assert_not_legacy_artifact(&legacy_path, true).is_err());
        assert!(assert_not_legacy_artifact(&legacy_path, false).is_ok());
    }

    #[test]
    fn guard_allows_execution_log() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let log_path = root.join(".musketeer/runs/test-id/execution-log.yml");
        assert!(assert_not_legacy_artifact(&log_path, true).is_ok());
    }

    #[test]
    fn ensure_dirs_create() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        // State dir doesn't exist yet
        std::fs::create_dir_all(layout::state_dir(root)).unwrap();
        ensure_packets_dir(root).unwrap();
        ensure_verdicts_dir(root).unwrap();
        ensure_runs_dir(root).unwrap();
        ensure_bridge_dir(root).unwrap();
        assert!(packets_dir(root).is_dir());
        assert!(verdicts_dir(root).is_dir());
        assert!(runs_dir(root).is_dir());
        assert!(bridge_dir(root).is_dir());
    }
}
