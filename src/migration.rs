//! Migration engine: legacy workspace -> SMALL-native.
//!
//! Converts `.musketeer/runs/<id>/` legacy artifacts into `.small/` canonical
//! artifacts while archiving originals under `.musketeer/legacy/`.

use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::fs::layout;
use crate::model::verdict::Verdict;
use crate::small_workspace;

// ---------------------------------------------------------------------------
// Migration state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationState {
    Legacy,
    SmallNative,
    Mixed,
    Empty,
}

pub fn detect_migration_state(root: &Path) -> MigrationState {
    let has_small = small_workspace::exists(root);
    let has_legacy_runs = layout::runs_dir(root).is_dir() && has_legacy_artifacts(root);
    match (has_small, has_legacy_runs) {
        (true, true) => MigrationState::Mixed,
        (true, false) => MigrationState::SmallNative,
        (false, true) => MigrationState::Legacy,
        (false, false) => MigrationState::Empty,
    }
}

fn has_legacy_artifacts(root: &Path) -> bool {
    let runs = layout::runs_dir(root);
    if let Ok(entries) = fs::read_dir(&runs) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let dir = entry.path();
                for name in LEGACY_ARTIFACT_NAMES {
                    if dir.join(name).is_file() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

const LEGACY_ARTIFACT_NAMES: &[&str] = &[
    "intent.yml",
    "constraints.yml",
    "plan.yml",
    "progress.yml",
    "handoff.yml",
];

// ---------------------------------------------------------------------------
// Migration plan
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPlan {
    pub replay_id: String,
    pub all_run_ids: Vec<String>,
    pub artifacts_found: Vec<ArtifactRef>,
    pub archive_timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactRef {
    pub run_id: String,
    pub filename: String,
    pub path: String,
}

pub fn plan_migration(root: &Path) -> anyhow::Result<MigrationPlan> {
    let runs_dir = layout::runs_dir(root);
    let mut run_ids: Vec<String> = fs::read_dir(&runs_dir)
        .context("failed to read runs dir")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    run_ids.sort();

    if run_ids.is_empty() {
        anyhow::bail!("no legacy runs found in .musketeer/runs/");
    }

    let replay_id = run_ids.last().unwrap().clone();

    let mut artifacts_found = Vec::new();
    for run_id in &run_ids {
        let run_dir = layout::run_dir(root, run_id);
        for name in LEGACY_ARTIFACT_NAMES {
            let path = run_dir.join(name);
            if path.is_file() {
                artifacts_found.push(ArtifactRef {
                    run_id: run_id.clone(),
                    filename: name.to_string(),
                    path: format!(".musketeer/runs/{}/{}", run_id, name),
                });
            }
        }
    }

    let now = OffsetDateTime::now_utc();
    let archive_timestamp = now
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
        .replace(':', "-");

    Ok(MigrationPlan {
        replay_id,
        all_run_ids: run_ids,
        artifacts_found,
        archive_timestamp,
    })
}

// ---------------------------------------------------------------------------
// Migration report
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub kind: String,
    pub timestamp: String,
    pub source_mode: String,
    pub target_mode: String,
    pub replay_id: String,
    pub files_found: Vec<String>,
    pub files_converted: Vec<FileConversion>,
    pub files_archived: Vec<String>,
    pub fields_ambiguous: Vec<AmbiguousField>,
    pub warnings: Vec<String>,
    pub other_runs_archived: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileConversion {
    pub source: String,
    pub target: String,
    pub fields_mapped: Vec<String>,
    pub fields_dropped: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmbiguousField {
    pub source: String,
    pub field: String,
    pub reason: String,
    pub action: String,
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

pub fn execute_migration(root: &Path, plan: &MigrationPlan) -> anyhow::Result<MigrationReport> {
    let now = OffsetDateTime::now_utc();
    let timestamp = now
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    // Step 3: Archive originals
    archive_legacy(root, &plan.archive_timestamp)?;

    // Step 4: Convert artifacts for the active run
    let conversion = convert_artifacts(root, &plan.replay_id)?;

    // Step 5: Restructure .musketeer/
    restructure_musketeer(root, &plan.replay_id)?;

    // Remove legacy artifacts from active run dir (already archived)
    let run_dir = layout::run_dir(root, &plan.replay_id);
    for name in LEGACY_ARTIFACT_NAMES {
        let p = run_dir.join(name);
        if p.is_file() {
            fs::remove_file(&p).ok();
        }
    }
    // Also remove legacy artifacts from other runs
    for run_id in &plan.all_run_ids {
        if run_id != &plan.replay_id {
            let other_run = layout::run_dir(root, run_id);
            for name in LEGACY_ARTIFACT_NAMES {
                let p = other_run.join(name);
                if p.is_file() {
                    fs::remove_file(&p).ok();
                }
            }
            // Remove empty run dirs
            if other_run.is_dir() {
                let _ = fs::remove_dir(&other_run); // only removes if empty
            }
        }
    }

    let files_found: Vec<String> = plan.artifacts_found.iter().map(|a| a.path.clone()).collect();
    let files_archived: Vec<String> = plan
        .artifacts_found
        .iter()
        .map(|a| {
            format!(
                ".musketeer/legacy/{}/runs/{}/{}",
                plan.archive_timestamp, a.run_id, a.filename
            )
        })
        .collect();

    let other_runs: Vec<String> = plan
        .all_run_ids
        .iter()
        .filter(|id| *id != &plan.replay_id)
        .cloned()
        .collect();

    let mut warnings = Vec::new();
    if !other_runs.is_empty() {
        warnings.push(format!(
            "{} additional run(s) archived but not converted: {}. Review manually if needed.",
            other_runs.len(),
            other_runs.join(", ")
        ));
    }

    let report = MigrationReport {
        kind: "musketeer_migration_report".to_string(),
        timestamp,
        source_mode: "legacy".to_string(),
        target_mode: "small_native".to_string(),
        replay_id: plan.replay_id.clone(),
        files_found,
        files_converted: conversion.files_converted,
        files_archived,
        fields_ambiguous: conversion.fields_ambiguous,
        warnings,
        other_runs_archived: other_runs,
    };

    // Write report
    let report_path = layout::state_dir(root).join("migration-report.yml");
    let yaml = serde_yaml::to_string(&report).context("failed to serialize report")?;
    crate::fs::write::write_file_atomic(&report_path, yaml.as_bytes())?;

    Ok(report)
}

pub fn archive_legacy(root: &Path, timestamp: &str) -> anyhow::Result<()> {
    let archive_dir = layout::state_dir(root).join("legacy").join(timestamp);
    let runs_src = layout::runs_dir(root);

    if !runs_src.is_dir() {
        return Ok(());
    }

    let archive_runs = archive_dir.join("runs");
    copy_dir_recursive(&runs_src, &archive_runs)?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Artifact conversion
// ---------------------------------------------------------------------------

pub struct ConversionResult {
    pub files_converted: Vec<FileConversion>,
    pub fields_ambiguous: Vec<AmbiguousField>,
}

pub fn convert_artifacts(root: &Path, replay_id: &str) -> anyhow::Result<ConversionResult> {
    let small_dir = small_workspace::small_dir(root);
    fs::create_dir_all(&small_dir)?;

    let run_dir = layout::run_dir(root, replay_id);
    let mut files_converted = Vec::new();
    let mut fields_ambiguous = Vec::new();

    // workspace.small.yml
    let ws_content = format!("replay_id: {}\nversion: 1\n", replay_id);
    fs::write(small_workspace::workspace_path(root), &ws_content)?;

    // intent
    let intent_src = run_dir.join("intent.yml");
    if intent_src.is_file() {
        let raw: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&intent_src)?)?;
        let mut out = serde_yaml::Mapping::new();
        if let Some(v) = raw.get("title") {
            out.insert("title".into(), v.clone());
        }
        if let Some(v) = raw.get("outcome") {
            out.insert("outcome".into(), v.clone());
        }
        let yaml = serde_yaml::to_string(&out)?;
        fs::write(small_workspace::intent_path(root), &yaml)?;
        files_converted.push(FileConversion {
            source: format!(".musketeer/runs/{}/intent.yml", replay_id),
            target: ".small/intent.small.yml".to_string(),
            fields_mapped: vec!["title".into(), "outcome".into()],
            fields_dropped: vec!["replay_id".into()],
        });
    }

    // constraints
    let constraints_src = run_dir.join("constraints.yml");
    if constraints_src.is_file() {
        let raw: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&constraints_src)?)?;
        let mut out = serde_yaml::Mapping::new();
        if let Some(v) = raw.get("scope") {
            out.insert("scope".into(), v.clone());
        }
        if let Some(v) = raw.get("non_goals") {
            out.insert("non_goals".into(), v.clone());
        }
        let yaml = serde_yaml::to_string(&out)?;
        fs::write(small_workspace::constraints_path(root), &yaml)?;

        let dropped = vec!["replay_id".to_string()];
        if raw.get("allowlist").is_some() {
            fields_ambiguous.push(AmbiguousField {
                source: "constraints.yml".to_string(),
                field: "allowlist".to_string(),
                reason: "no equivalent in SMALL constraints schema".to_string(),
                action: "preserved in archive only".to_string(),
            });
        }
        files_converted.push(FileConversion {
            source: format!(".musketeer/runs/{}/constraints.yml", replay_id),
            target: ".small/constraints.small.yml".to_string(),
            fields_mapped: vec!["scope".into(), "non_goals".into()],
            fields_dropped: dropped,
        });
    }

    // plan
    let plan_src = run_dir.join("plan.yml");
    if plan_src.is_file() {
        let raw: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&plan_src)?)?;
        let mut out = serde_yaml::Mapping::new();
        if let Some(v) = raw.get("tasks") {
            out.insert("tasks".into(), v.clone());
        }
        let yaml = serde_yaml::to_string(&out)?;
        fs::write(small_workspace::plan_path(root), &yaml)?;
        files_converted.push(FileConversion {
            source: format!(".musketeer/runs/{}/plan.yml", replay_id),
            target: ".small/plan.small.yml".to_string(),
            fields_mapped: vec!["tasks".into()],
            fields_dropped: vec!["replay_id".into()],
        });
    }

    // progress
    let progress_src = run_dir.join("progress.yml");
    if progress_src.is_file() {
        let raw: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&progress_src)?)?;
        let mut out = serde_yaml::Mapping::new();
        if let Some(v) = raw.get("entries") {
            out.insert("entries".into(), v.clone());
        }
        let yaml = serde_yaml::to_string(&out)?;
        fs::write(small_workspace::progress_path(root), &yaml)?;
        files_converted.push(FileConversion {
            source: format!(".musketeer/runs/{}/progress.yml", replay_id),
            target: ".small/progress.small.yml".to_string(),
            fields_mapped: vec!["entries".into()],
            fields_dropped: vec!["replay_id".into()],
        });
    }

    // handoff
    let handoff_src = run_dir.join("handoff.yml");
    if handoff_src.is_file() {
        let raw: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&handoff_src)?)?;
        let mut out = serde_yaml::Mapping::new();
        if let Some(v) = raw.get("note") {
            out.insert("note".into(), v.clone());
        }
        let yaml = serde_yaml::to_string(&out)?;
        fs::write(small_workspace::handoff_path(root), &yaml)?;
        files_converted.push(FileConversion {
            source: format!(".musketeer/runs/{}/handoff.yml", replay_id),
            target: ".small/handoff.small.yml".to_string(),
            fields_mapped: vec!["note".into()],
            fields_dropped: vec!["replay_id".into()],
        });

        // Extract verdict if present
        let verdict_val = raw.get("verdict").and_then(|v| v.as_str().map(|s| s.to_string()));
        let verdict_reason = raw
            .get("verdict_reason")
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        if let Some(verdict_status) = verdict_val {
            let verdicts_dir = layout::state_dir(root).join("verdicts");
            fs::create_dir_all(&verdicts_dir)?;
            let now = OffsetDateTime::now_utc();
            let ts = now.format(&Rfc3339).unwrap_or_else(|_| "unknown".into());
            let verdict = Verdict {
                kind: "musketeer_verdict".to_string(),
                musketeer_version: env!("CARGO_PKG_VERSION").to_string(),
                replay_id: replay_id.to_string(),
                status: verdict_status,
                gate: "auditor".to_string(),
                reason: verdict_reason.unwrap_or_default(),
                timestamp: ts,
            };
            let verdict_path = verdicts_dir.join(format!("{}.verdict.yml", replay_id));
            let verdict_yaml = serde_yaml::to_string(&verdict)?;
            fs::write(&verdict_path, &verdict_yaml)?;
        }
    }

    Ok(ConversionResult {
        files_converted,
        fields_ambiguous,
    })
}

fn restructure_musketeer(root: &Path, replay_id: &str) -> anyhow::Result<()> {
    let state = layout::state_dir(root);
    fs::create_dir_all(state.join("packets"))?;
    fs::create_dir_all(state.join("verdicts"))?;

    let run_dir = layout::run_dir(root, replay_id);
    fs::create_dir_all(&run_dir)?;

    // If progress had entries, create execution-log.yml
    let progress_src = layout::progress_path(root, replay_id);
    if progress_src.is_file() {
        let raw: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&progress_src)?)?;
        if let Some(entries) = raw.get("entries") {
            let mut log = serde_yaml::Mapping::new();
            log.insert("replay_id".into(), replay_id.into());
            log.insert("entries".into(), entries.clone());
            let yaml = serde_yaml::to_string(&log)?;
            let exec_log_path = run_dir.join("execution-log.yml");
            fs::write(&exec_log_path, &yaml)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_legacy_workspace(tmp: &TempDir, replay_id: &str) {
        let run_dir = layout::run_dir(tmp.path(), replay_id);
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(
            run_dir.join("intent.yml"),
            format!("replay_id: {}\ntitle: Test\noutcome: Done\n", replay_id),
        )
        .unwrap();
        fs::write(
            run_dir.join("constraints.yml"),
            format!(
                "replay_id: {}\nscope:\n  - src/\nnon_goals:\n  - docs/\nallowlist:\n  - foo\n",
                replay_id
            ),
        )
        .unwrap();
        fs::write(
            run_dir.join("plan.yml"),
            format!(
                "replay_id: {}\ntasks:\n  - id: t1\n    title: Task 1\n    status: pending\n",
                replay_id
            ),
        )
        .unwrap();
        fs::write(
            run_dir.join("progress.yml"),
            format!(
                "replay_id: {}\nentries:\n  - seq: 1\n    ts: '2026-01-01T00:00:00Z'\n    role: executor\n    kind: note\n    message: started\n    summary: started\n",
                replay_id
            ),
        )
        .unwrap();
        fs::write(
            run_dir.join("handoff.yml"),
            format!("replay_id: {}\nnote: Ready\n", replay_id),
        )
        .unwrap();
    }

    #[test]
    fn detect_legacy_state() {
        let tmp = TempDir::new().unwrap();
        create_legacy_workspace(&tmp, "run-001");
        assert_eq!(detect_migration_state(tmp.path()), MigrationState::Legacy);
    }

    #[test]
    fn detect_small_native_state() {
        let tmp = TempDir::new().unwrap();
        let dir = small_workspace::small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("workspace.small.yml"), "version: 1\n").unwrap();
        assert_eq!(
            detect_migration_state(tmp.path()),
            MigrationState::SmallNative
        );
    }

    #[test]
    fn detect_empty_state() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_migration_state(tmp.path()), MigrationState::Empty);
    }

    #[test]
    fn detect_mixed_state() {
        let tmp = TempDir::new().unwrap();
        create_legacy_workspace(&tmp, "run-001");
        let dir = small_workspace::small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("workspace.small.yml"), "version: 1\n").unwrap();
        assert_eq!(detect_migration_state(tmp.path()), MigrationState::Mixed);
    }

    #[test]
    fn full_migration_legacy_to_small() {
        let tmp = TempDir::new().unwrap();
        create_legacy_workspace(&tmp, "run-001");
        let plan = plan_migration(tmp.path()).unwrap();
        assert_eq!(plan.replay_id, "run-001");
        assert_eq!(plan.artifacts_found.len(), 5);

        let report = execute_migration(tmp.path(), &plan).unwrap();
        assert_eq!(report.source_mode, "legacy");
        assert_eq!(report.target_mode, "small_native");

        // .small/ created
        assert!(small_workspace::small_dir(tmp.path()).is_dir());
        assert!(small_workspace::workspace_path(tmp.path()).is_file());
        assert!(small_workspace::intent_path(tmp.path()).is_file());

        // Archive created
        let archive = layout::state_dir(tmp.path())
            .join("legacy")
            .join(&plan.archive_timestamp)
            .join("runs")
            .join("run-001")
            .join("intent.yml");
        assert!(archive.is_file());

        // Legacy artifacts removed from run dir
        assert!(!layout::intent_path(tmp.path(), "run-001").is_file());

        // Migration report written
        assert!(layout::state_dir(tmp.path())
            .join("migration-report.yml")
            .is_file());

        // Workspace now detects as SmallNative
        assert_eq!(
            detect_migration_state(tmp.path()),
            MigrationState::SmallNative
        );

        // Ambiguous field reported
        assert_eq!(report.fields_ambiguous.len(), 1);
        assert_eq!(report.fields_ambiguous[0].field, "allowlist");
    }

    #[test]
    fn already_small_native() {
        let tmp = TempDir::new().unwrap();
        let dir = small_workspace::small_dir(tmp.path());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("workspace.small.yml"), "version: 1\n").unwrap();
        assert_eq!(
            detect_migration_state(tmp.path()),
            MigrationState::SmallNative
        );
    }

    #[test]
    fn empty_workspace_no_plan() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_migration_state(tmp.path()), MigrationState::Empty);
    }

    #[test]
    fn dry_run_creates_no_files() {
        let tmp = TempDir::new().unwrap();
        create_legacy_workspace(&tmp, "run-001");
        let plan = plan_migration(tmp.path()).unwrap();
        // Dry run = just plan, no execute. Verify .small/ does NOT exist.
        assert!(!small_workspace::small_dir(tmp.path()).is_dir());
        // Plan is valid
        assert_eq!(plan.replay_id, "run-001");
    }

    #[test]
    fn verdict_extraction() {
        let tmp = TempDir::new().unwrap();
        let run_dir = layout::run_dir(tmp.path(), "run-v");
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(
            run_dir.join("intent.yml"),
            "replay_id: run-v\ntitle: T\noutcome: O\n",
        )
        .unwrap();
        fs::write(
            run_dir.join("constraints.yml"),
            "replay_id: run-v\nscope:\n  - src/\nnon_goals: []\n",
        )
        .unwrap();
        fs::write(
            run_dir.join("plan.yml"),
            "replay_id: run-v\ntasks: []\n",
        )
        .unwrap();
        fs::write(
            run_dir.join("progress.yml"),
            "replay_id: run-v\nentries: []\n",
        )
        .unwrap();
        fs::write(
            run_dir.join("handoff.yml"),
            "replay_id: run-v\nnote: Done\nverdict: approve\nverdict_reason: looks good\n",
        )
        .unwrap();

        let plan = plan_migration(tmp.path()).unwrap();
        execute_migration(tmp.path(), &plan).unwrap();

        let verdict_path = layout::state_dir(tmp.path())
            .join("verdicts")
            .join("run-v.verdict.yml");
        assert!(verdict_path.is_file());
        let content = fs::read_to_string(&verdict_path).unwrap();
        assert!(content.contains("approve"));
    }

    #[test]
    fn multiple_runs_most_recent_active() {
        let tmp = TempDir::new().unwrap();
        create_legacy_workspace(&tmp, "run-001");
        create_legacy_workspace(&tmp, "run-002");

        let plan = plan_migration(tmp.path()).unwrap();
        assert_eq!(plan.replay_id, "run-002");
        assert_eq!(plan.all_run_ids, vec!["run-001", "run-002"]);

        let report = execute_migration(tmp.path(), &plan).unwrap();
        assert_eq!(report.replay_id, "run-002");
        assert_eq!(report.other_runs_archived, vec!["run-001"]);
        assert!(!report.warnings.is_empty());

        // Workspace file references the latest run
        let ws = fs::read_to_string(small_workspace::workspace_path(tmp.path())).unwrap();
        assert!(ws.contains("run-002"));
    }
}
