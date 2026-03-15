//! Replay ID source resolution.
//!
//! In a SMALL-native workspace the replay ID is sourced from the SMALL
//! workspace file. This module resolves the canonical replay ID and
//! rejects conflicting legacy identities.

use std::path::Path;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::fs::read;
use crate::legacy_workspace::{self, WorkspaceMode};
use crate::small_workspace;

/// A minimal struct for reading replay_id from workspace.small.yml.
#[derive(Debug, serde::Deserialize)]
struct SmallWorkspacePartial {
    #[serde(default)]
    replay_id: Option<String>,
}

/// Resolve the canonical replay ID from a SMALL workspace.
///
/// Returns `None` if the workspace file does not contain a replay_id field.
/// Returns an error if the workspace file cannot be read or parsed.
pub fn resolve_from_small(root: &Path) -> anyhow::Result<Option<String>> {
    let ws_path = small_workspace::workspace_path(root);
    if !ws_path.is_file() {
        return Ok(None);
    }
    let partial: SmallWorkspacePartial =
        read::read_yaml(&ws_path).context("failed to read workspace.small.yml")?;
    Ok(partial.replay_id)
}

/// Reject if a legacy replay ID conflicts with the SMALL-sourced one.
pub fn reject_conflict(
    small_replay_id: &str,
    legacy_replay_id: &str,
) -> Result<(), MusketeerError> {
    if small_replay_id != legacy_replay_id {
        return Err(MusketeerError::InvalidInput(format!(
            "replay ID mismatch: SMALL workspace says '{}' but legacy says '{}'",
            small_replay_id, legacy_replay_id
        )));
    }
    Ok(())
}

/// High-level: get the replay ID for the current workspace, preferring SMALL.
pub fn canonical_replay_id(root: &Path) -> anyhow::Result<Option<String>> {
    let mode = legacy_workspace::detect(root);
    match mode {
        WorkspaceMode::SmallNative => resolve_from_small(root),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_from_missing_workspace() {
        let tmp = TempDir::new().unwrap();
        assert!(resolve_from_small(tmp.path()).unwrap().is_none());
    }

    #[test]
    fn resolve_from_workspace_with_replay_id() {
        let tmp = TempDir::new().unwrap();
        let dir = small_workspace::small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            small_workspace::workspace_path(tmp.path()),
            "replay_id: abc-123\n",
        )
        .unwrap();
        let id = resolve_from_small(tmp.path()).unwrap();
        assert_eq!(id.as_deref(), Some("abc-123"));
    }

    #[test]
    fn conflict_rejected() {
        let err = reject_conflict("aaa", "bbb");
        assert!(err.is_err());
    }

    #[test]
    fn no_conflict() {
        assert!(reject_conflict("same", "same").is_ok());
    }
}
