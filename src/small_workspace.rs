//! SMALL workspace detection and canonical artifact path helpers.
//!
//! A SMALL workspace is identified by the presence of a `.small/` directory
//! at the project root. Canonical artifacts live inside `.small/` and are
//! owned by the SMALL protocol -- Musketeer must not write to them.

use std::path::{Path, PathBuf};

pub const SMALL_DIR: &str = ".small";

pub const WORKSPACE_FILE: &str = "workspace.small.yml";
pub const INTENT_FILE: &str = "intent.small.yml";
pub const CONSTRAINTS_FILE: &str = "constraints.small.yml";
pub const PLAN_FILE: &str = "plan.small.yml";
pub const PROGRESS_FILE: &str = "progress.small.yml";
pub const HANDOFF_FILE: &str = "handoff.small.yml";

/// All canonical SMALL artifact filenames (excluding workspace).
pub const CANONICAL_ARTIFACTS: &[&str] = &[
    INTENT_FILE,
    CONSTRAINTS_FILE,
    PLAN_FILE,
    PROGRESS_FILE,
    HANDOFF_FILE,
];

/// Root `.small/` directory path.
pub fn small_dir(root: &Path) -> PathBuf {
    root.join(SMALL_DIR)
}

pub fn workspace_path(root: &Path) -> PathBuf {
    small_dir(root).join(WORKSPACE_FILE)
}

pub fn intent_path(root: &Path) -> PathBuf {
    small_dir(root).join(INTENT_FILE)
}

pub fn constraints_path(root: &Path) -> PathBuf {
    small_dir(root).join(CONSTRAINTS_FILE)
}

pub fn plan_path(root: &Path) -> PathBuf {
    small_dir(root).join(PLAN_FILE)
}

pub fn progress_path(root: &Path) -> PathBuf {
    small_dir(root).join(PROGRESS_FILE)
}

pub fn handoff_path(root: &Path) -> PathBuf {
    small_dir(root).join(HANDOFF_FILE)
}

/// Returns true if a `.small/` directory exists at the given root.
pub fn exists(root: &Path) -> bool {
    small_dir(root).is_dir()
}

/// Returns true if the workspace file and all five canonical artifacts exist.
pub fn is_valid(root: &Path) -> bool {
    if !exists(root) {
        return false;
    }
    if !workspace_path(root).is_file() {
        return false;
    }
    CANONICAL_ARTIFACTS.iter().all(|name| small_dir(root).join(name).is_file())
}

/// Returns a list of missing canonical artifacts (empty if all present).
pub fn missing_artifacts(root: &Path) -> Vec<&'static str> {
    let dir = small_dir(root);
    CANONICAL_ARTIFACTS
        .iter()
        .filter(|name| !dir.join(name).is_file())
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_missing_small_dir() {
        let tmp = TempDir::new().unwrap();
        assert!(!exists(tmp.path()));
        assert!(!is_valid(tmp.path()));
    }

    #[test]
    fn detects_present_small_dir() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(small_dir(tmp.path())).unwrap();
        assert!(exists(tmp.path()));
        // Not valid yet -- no artifacts
        assert!(!is_valid(tmp.path()));
    }

    #[test]
    fn valid_when_all_artifacts_present() {
        let tmp = TempDir::new().unwrap();
        let dir = small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(WORKSPACE_FILE), "version: 1\n").unwrap();
        for name in CANONICAL_ARTIFACTS {
            fs::write(dir.join(name), "kind: test\n").unwrap();
        }
        assert!(is_valid(tmp.path()));
        assert!(missing_artifacts(tmp.path()).is_empty());
    }

    #[test]
    fn reports_missing_artifacts() {
        let tmp = TempDir::new().unwrap();
        let dir = small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(WORKSPACE_FILE), "version: 1\n").unwrap();
        fs::write(dir.join(INTENT_FILE), "kind: intent\n").unwrap();
        let missing = missing_artifacts(tmp.path());
        assert_eq!(missing.len(), 4);
        assert!(!missing.contains(&INTENT_FILE));
    }
}
