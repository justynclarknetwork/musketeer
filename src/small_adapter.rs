//! SMALL artifact adapter -- the single translation seam.
//!
//! This module defines read-only SMALL artifact shapes, loads them from
//! `.small/*.small.yml`, and maps them to internal Musketeer model types.
//! Musketeer NEVER writes to `.small/`.

use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::fs::read;
use crate::model::progress::{ProgressEntry, ProgressLog};
use crate::model::run::{Constraints, Handoff, Intent, Plan, PlanTask};
use crate::small_workspace;

// ---------------------------------------------------------------------------
// SMALL canonical artifact shapes (read-only from .small/)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SmallIntent {
    pub title: String,
    pub outcome: String,
}

#[derive(Debug, Deserialize)]
pub struct SmallConstraints {
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SmallPlan {
    pub tasks: Vec<SmallPlanTask>,
}

#[derive(Debug, Deserialize)]
pub struct SmallPlanTask {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SmallProgress {
    pub entries: Vec<SmallProgressEntry>,
}

#[derive(Debug, Deserialize)]
pub struct SmallProgressEntry {
    pub seq: u64,
    pub ts: String,
    pub role: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct SmallHandoff {
    pub note: String,
}

// ---------------------------------------------------------------------------
// Loaders -- read from .small/*.small.yml
// ---------------------------------------------------------------------------

pub fn load_intent(root: &Path) -> anyhow::Result<SmallIntent> {
    read::read_yaml(&small_workspace::intent_path(root))
        .context("failed to load .small/intent.small.yml")
}

pub fn load_constraints(root: &Path) -> anyhow::Result<SmallConstraints> {
    read::read_yaml(&small_workspace::constraints_path(root))
        .context("failed to load .small/constraints.small.yml")
}

pub fn load_plan(root: &Path) -> anyhow::Result<SmallPlan> {
    read::read_yaml(&small_workspace::plan_path(root))
        .context("failed to load .small/plan.small.yml")
}

pub fn load_progress(root: &Path) -> anyhow::Result<SmallProgress> {
    read::read_yaml(&small_workspace::progress_path(root))
        .context("failed to load .small/progress.small.yml")
}

pub fn load_handoff(root: &Path) -> anyhow::Result<SmallHandoff> {
    read::read_yaml(&small_workspace::handoff_path(root))
        .context("failed to load .small/handoff.small.yml")
}

// ---------------------------------------------------------------------------
// Mappers -- SMALL artifact -> internal Musketeer model
// ---------------------------------------------------------------------------

pub fn map_intent(small: SmallIntent, replay_id: &str) -> Intent {
    Intent {
        replay_id: replay_id.to_string(),
        title: small.title,
        outcome: small.outcome,
    }
}

pub fn map_constraints(small: SmallConstraints, replay_id: &str) -> Constraints {
    Constraints {
        replay_id: replay_id.to_string(),
        scope: small.scope,
        non_goals: small.non_goals,
        allowlist: Vec::new(), // SMALL does not define allowlist; Musketeer policy owns it
    }
}

pub fn map_plan(small: SmallPlan, replay_id: &str) -> Plan {
    Plan {
        replay_id: replay_id.to_string(),
        tasks: small
            .tasks
            .into_iter()
            .map(|t| PlanTask {
                id: t.id,
                title: t.title,
                status: t.status,
            })
            .collect(),
    }
}

pub fn map_progress(small: SmallProgress, replay_id: &str) -> ProgressLog {
    ProgressLog {
        replay_id: replay_id.to_string(),
        entries: small
            .entries
            .into_iter()
            .map(|e| ProgressEntry {
                seq: e.seq,
                ts: e.ts,
                role: e.role,
                kind: e.kind,
                message: e.message,
                summary: e.summary,
            })
            .collect(),
    }
}

pub fn map_handoff(small: SmallHandoff, replay_id: &str) -> Handoff {
    Handoff {
        replay_id: replay_id.to_string(),
        note: small.note,
        verdict: None,
        verdict_reason: None,
    }
}

// ---------------------------------------------------------------------------
// Convenience: load + map in one call
// ---------------------------------------------------------------------------

pub fn read_intent(root: &Path, replay_id: &str) -> anyhow::Result<Intent> {
    load_intent(root).map(|s| map_intent(s, replay_id))
}

pub fn read_constraints(root: &Path, replay_id: &str) -> anyhow::Result<Constraints> {
    load_constraints(root).map(|s| map_constraints(s, replay_id))
}

pub fn read_plan(root: &Path, replay_id: &str) -> anyhow::Result<Plan> {
    load_plan(root).map(|s| map_plan(s, replay_id))
}

pub fn read_progress(root: &Path, replay_id: &str) -> anyhow::Result<ProgressLog> {
    load_progress(root).map(|s| map_progress(s, replay_id))
}

pub fn read_handoff(root: &Path, replay_id: &str) -> anyhow::Result<Handoff> {
    load_handoff(root).map(|s| map_handoff(s, replay_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_small_artifacts(tmp: &TempDir) {
        let dir = small_workspace::small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            small_workspace::intent_path(tmp.path()),
            "title: Test Intent\noutcome: Test Outcome\n",
        )
        .unwrap();
        fs::write(
            small_workspace::constraints_path(tmp.path()),
            "scope:\n  - src/\nnon_goals:\n  - docs/\n",
        )
        .unwrap();
        fs::write(
            small_workspace::plan_path(tmp.path()),
            "tasks:\n  - id: t1\n    title: Task One\n    status: pending\n",
        )
        .unwrap();
        fs::write(
            small_workspace::progress_path(tmp.path()),
            "entries:\n  - seq: 1\n    ts: '2026-01-01T00:00:00Z'\n    role: executor\n    kind: note\n    message: started\n    summary: started\n",
        )
        .unwrap();
        fs::write(
            small_workspace::handoff_path(tmp.path()),
            "note: Ready for review\n",
        )
        .unwrap();
    }

    #[test]
    fn loads_and_maps_intent() {
        let tmp = TempDir::new().unwrap();
        setup_small_artifacts(&tmp);
        let intent = read_intent(tmp.path(), "replay-1").unwrap();
        assert_eq!(intent.title, "Test Intent");
        assert_eq!(intent.outcome, "Test Outcome");
        assert_eq!(intent.replay_id, "replay-1");
    }

    #[test]
    fn loads_and_maps_constraints() {
        let tmp = TempDir::new().unwrap();
        setup_small_artifacts(&tmp);
        let c = read_constraints(tmp.path(), "replay-1").unwrap();
        assert_eq!(c.scope, vec!["src/"]);
        assert_eq!(c.non_goals, vec!["docs/"]);
        assert!(c.allowlist.is_empty());
    }

    #[test]
    fn loads_and_maps_plan() {
        let tmp = TempDir::new().unwrap();
        setup_small_artifacts(&tmp);
        let plan = read_plan(tmp.path(), "replay-1").unwrap();
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].id, "t1");
    }

    #[test]
    fn loads_and_maps_progress() {
        let tmp = TempDir::new().unwrap();
        setup_small_artifacts(&tmp);
        let progress = read_progress(tmp.path(), "replay-1").unwrap();
        assert_eq!(progress.entries.len(), 1);
        assert_eq!(progress.entries[0].seq, 1);
    }

    #[test]
    fn loads_and_maps_handoff() {
        let tmp = TempDir::new().unwrap();
        setup_small_artifacts(&tmp);
        let handoff = read_handoff(tmp.path(), "replay-1").unwrap();
        assert_eq!(handoff.note, "Ready for review");
        assert!(handoff.verdict.is_none());
    }

    #[test]
    fn missing_artifact_fails_clearly() {
        let tmp = TempDir::new().unwrap();
        // No .small/ directory at all
        assert!(load_intent(tmp.path()).is_err());
    }
}
