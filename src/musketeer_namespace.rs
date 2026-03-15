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

/// Path for a verdict file: `.musketeer/verdicts/<replayId>.verdict.yml`
pub fn verdict_path(root: &Path, replay_id: &str) -> PathBuf {
    verdicts_dir(root).join(format!("{}.verdict.yml", replay_id))
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

        assert_eq!(
            packets_dir(root),
            root.join(".musketeer").join("packets")
        );
        assert_eq!(
            verdicts_dir(root),
            root.join(".musketeer").join("verdicts")
        );
        assert_eq!(
            verdict_path(root, "abc-123"),
            root.join(".musketeer").join("verdicts").join("abc-123.verdict.yml")
        );
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
