//! Legacy workspace detection.
//!
//! Distinguishes between SMALL-native workspaces, legacy Musketeer-only
//! workspaces, and empty (uninitialized) directories.

use std::path::Path;

use crate::fs::layout;
use crate::small_workspace;

/// The detected workspace mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceMode {
    /// A `.small/` directory exists with canonical artifacts.
    SmallNative,
    /// Only `.musketeer/runs/` with old-style artifacts exists.
    Legacy,
    /// No workspace detected.
    Empty,
}

/// Detect the workspace mode at the given root.
pub fn detect(root: &Path) -> WorkspaceMode {
    if small_workspace::exists(root) {
        return WorkspaceMode::SmallNative;
    }
    if layout::runs_dir(root).is_dir() {
        return WorkspaceMode::Legacy;
    }
    WorkspaceMode::Empty
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn empty_dir_is_empty() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect(tmp.path()), WorkspaceMode::Empty);
    }

    #[test]
    fn legacy_detected() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(layout::runs_dir(tmp.path())).unwrap();
        assert_eq!(detect(tmp.path()), WorkspaceMode::Legacy);
    }

    #[test]
    fn small_native_detected() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(small_workspace::small_dir(tmp.path())).unwrap();
        // Also create legacy dir -- SMALL takes precedence
        fs::create_dir_all(layout::runs_dir(tmp.path())).unwrap();
        assert_eq!(detect(tmp.path()), WorkspaceMode::SmallNative);
    }
}
