//! Workspace mode routing.
//!
//! Single entry point for commands to resolve the workspace context.
//! Returns a `WorkspaceContext` that tells the caller whether to read
//! from `.small/` (SMALL-native) or legacy `.musketeer/runs/` paths.

use std::path::{Path, PathBuf};

use crate::error::MusketeerError;
use crate::fs::layout;
use crate::legacy_workspace::{self, WorkspaceMode};
use crate::replay_source;
use crate::small_workspace;

/// Resolved workspace context for command dispatch.
#[derive(Debug)]
pub enum WorkspaceContext {
    /// SMALL workspace detected. Canonical artifacts live in `.small/`.
    SmallNative {
        root: PathBuf,
        small_root: PathBuf,
        replay_id: Option<String>,
        musketeer_root: PathBuf,
    },
    /// Legacy workspace only. Artifacts live in `.musketeer/runs/<id>/`.
    Legacy {
        root: PathBuf,
        musketeer_root: PathBuf,
    },
}

impl WorkspaceContext {
    pub fn root(&self) -> &Path {
        match self {
            WorkspaceContext::SmallNative { root, .. } => root,
            WorkspaceContext::Legacy { root, .. } => root,
        }
    }

    pub fn is_small(&self) -> bool {
        matches!(self, WorkspaceContext::SmallNative { .. })
    }
}

/// Resolve the workspace context at the given root.
///
/// Returns an error if no workspace is detected at all.
pub fn resolve(root: &Path) -> anyhow::Result<WorkspaceContext> {
    let mode = legacy_workspace::detect(root);
    match mode {
        WorkspaceMode::SmallNative => {
            let replay_id = replay_source::resolve_from_small(root)?;
            Ok(WorkspaceContext::SmallNative {
                root: root.to_path_buf(),
                small_root: small_workspace::small_dir(root),
                replay_id,
                musketeer_root: layout::state_dir(root),
            })
        }
        WorkspaceMode::Legacy => Ok(WorkspaceContext::Legacy {
            root: root.to_path_buf(),
            musketeer_root: layout::state_dir(root),
        }),
        WorkspaceMode::Empty => {
            Err(MusketeerError::WorkspaceMissing("no workspace detected".to_string()).into())
        }
    }
}

/// Resolve the effective replay ID, validating against SMALL workspace if present.
///
/// If `cli_replay` is provided and a SMALL workspace has a replay_id, they must match.
/// If `cli_replay` is None, returns SMALL replay_id (if any) or falls back to latest legacy.
pub fn resolve_replay_id(
    ctx: &WorkspaceContext,
    cli_replay: Option<String>,
) -> anyhow::Result<String> {
    match ctx {
        WorkspaceContext::SmallNative { replay_id, root, .. } => {
            match (replay_id, cli_replay) {
                (Some(small_id), Some(cli_id)) => {
                    if small_id != &cli_id {
                        return Err(MusketeerError::InvalidInput(format!(
                            "replay ID conflict: --replay '{}' vs SMALL workspace '{}'",
                            cli_id, small_id
                        ))
                        .into());
                    }
                    Ok(cli_id)
                }
                (Some(small_id), None) => Ok(small_id.clone()),
                (None, Some(cli_id)) => Ok(cli_id),
                (None, None) => {
                    // Fall back to latest legacy run dir
                    crate::commands::util::latest_replay_id(root)
                }
            }
        }
        WorkspaceContext::Legacy { root, .. } => {
            match cli_replay {
                Some(id) => Ok(id),
                None => crate::commands::util::latest_replay_id(root),
            }
        }
    }
}

/// Emit a deprecation warning to stderr for legacy workspace usage.
pub fn warn_legacy() {
    eprintln!("[deprecated] Legacy workspace detected. SMALL-native mode is preferred. Migration required.");
    eprintln!("[deprecated] Reading from legacy Musketeer paths. Migrate to a SMALL workspace (.small/).");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_small_workspace(tmp: &TempDir, replay_id: Option<&str>) {
        let dir = small_workspace::small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        let ws_content = match replay_id {
            Some(id) => format!("replay_id: {}\n", id),
            None => "version: 1\n".to_string(),
        };
        fs::write(small_workspace::workspace_path(tmp.path()), ws_content).unwrap();
        for name in small_workspace::CANONICAL_ARTIFACTS {
            fs::write(dir.join(name), "kind: test\n").unwrap();
        }
    }

    fn setup_legacy_workspace(tmp: &TempDir, replay_id: &str) {
        let run_dir = layout::run_dir(tmp.path(), replay_id);
        fs::create_dir_all(&run_dir).unwrap();
    }

    #[test]
    fn resolves_small_native() {
        let tmp = TempDir::new().unwrap();
        setup_small_workspace(&tmp, Some("test-123"));
        let ctx = resolve(tmp.path()).unwrap();
        assert!(ctx.is_small());
        if let WorkspaceContext::SmallNative { replay_id, .. } = &ctx {
            assert_eq!(replay_id.as_deref(), Some("test-123"));
        }
    }

    #[test]
    fn resolves_legacy() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_workspace(&tmp, "legacy-run");
        let ctx = resolve(tmp.path()).unwrap();
        assert!(!ctx.is_small());
    }

    #[test]
    fn empty_workspace_errors() {
        let tmp = TempDir::new().unwrap();
        assert!(resolve(tmp.path()).is_err());
    }

    #[test]
    fn replay_conflict_rejected() {
        let tmp = TempDir::new().unwrap();
        setup_small_workspace(&tmp, Some("small-id"));
        let ctx = resolve(tmp.path()).unwrap();
        let result = resolve_replay_id(&ctx, Some("different-id".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn replay_matches_ok() {
        let tmp = TempDir::new().unwrap();
        setup_small_workspace(&tmp, Some("same-id"));
        let ctx = resolve(tmp.path()).unwrap();
        let result = resolve_replay_id(&ctx, Some("same-id".to_string()));
        assert_eq!(result.unwrap(), "same-id");
    }
}
