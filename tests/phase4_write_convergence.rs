//! Phase 4 integration tests: write-path convergence for SMALL-native workspaces.
//!
//! Proves that NO legacy base artifacts (intent.yml, constraints.yml, plan.yml,
//! progress.yml, handoff.yml) are ever created under `.musketeer/runs/` in
//! SMALL-native mode.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const LEGACY_ARTIFACTS: &[&str] = &[
    "intent.yml",
    "constraints.yml",
    "plan.yml",
    "progress.yml",
    "handoff.yml",
];

fn musketeer_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("musketeer");
    path
}

fn run_cmd(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(musketeer_bin())
        .current_dir(dir)
        .args(args)
        .output()
        .expect("failed to run musketeer")
}

fn setup_small_workspace(root: &Path, replay_id: &str) {
    let small_dir = root.join(".small");
    fs::create_dir_all(&small_dir).unwrap();
    fs::write(
        small_dir.join("workspace.small.yml"),
        format!("replay_id: {}\n", replay_id),
    )
    .unwrap();
    fs::write(
        small_dir.join("intent.small.yml"),
        "title: Test Intent\noutcome: Test Outcome\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("constraints.small.yml"),
        "scope:\n  - src/\nnon_goals:\n  - vendor/\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("plan.small.yml"),
        "tasks:\n  - id: t1\n    title: Task One\n    status: pending\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("progress.small.yml"),
        "entries:\n  - seq: 1\n    ts: '2026-01-01T00:00:00Z'\n    role: executor\n    kind: note\n    message: init\n    summary: init\n",
    )
    .unwrap();
    fs::write(small_dir.join("handoff.small.yml"), "note: Ready\n").unwrap();
    fs::create_dir_all(root.join(".musketeer/runs")).unwrap();
}

/// Recursively scan a directory for any legacy artifact files.
fn find_legacy_artifacts(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    if !dir.exists() {
        return found;
    }
    for entry in walkdir(dir) {
        if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
            if LEGACY_ARTIFACTS.contains(&name) {
                found.push(entry.display().to_string());
            }
        }
    }
    found
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path));
            } else {
                results.push(path);
            }
        }
    }
    results
}

#[test]
fn run_new_creates_no_legacy_artifacts() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-new-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(tmp.path(), &["--json", "run", "new"]);
    assert!(
        out.status.success(),
        "run new failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let musketeer_dir = tmp.path().join(".musketeer");
    let legacy = find_legacy_artifacts(&musketeer_dir);
    assert!(
        legacy.is_empty(),
        "Legacy artifacts found after run new in SMALL-native mode: {:?}",
        legacy
    );
}

#[test]
fn log_writes_execution_log_not_legacy_progress() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-log-001";
    setup_small_workspace(tmp.path(), replay_id);

    // Create run dir
    fs::create_dir_all(tmp.path().join(format!(".musketeer/runs/{}", replay_id))).unwrap();

    let out = run_cmd(
        tmp.path(),
        &[
            "--json",
            "log",
            "--role",
            "executor",
            "--kind",
            "note",
            "--message",
            "test entry",
            "--replay",
            replay_id,
        ],
    );
    assert!(
        out.status.success(),
        "log failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Verify execution log exists
    let exec_log = tmp
        .path()
        .join(format!(".musketeer/runs/{}/execution-log.yml", replay_id));
    assert!(exec_log.is_file(), "execution-log.yml should exist");

    // Verify it has correct schema
    let content = fs::read_to_string(&exec_log).unwrap();
    assert!(
        content.contains("musketeer_execution_log"),
        "should have correct kind"
    );
    assert!(
        content.contains("test entry"),
        "should contain the logged message"
    );

    // Verify NO legacy artifacts
    let musketeer_dir = tmp.path().join(".musketeer");
    let legacy = find_legacy_artifacts(&musketeer_dir);
    assert!(
        legacy.is_empty(),
        "Legacy artifacts found after log in SMALL-native mode: {:?}",
        legacy
    );
}

#[test]
fn verdict_writes_to_musketeer_verdicts_not_legacy() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-verdict-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(
        tmp.path(),
        &[
            "--json",
            "verdict",
            "--role",
            "auditor",
            "--value",
            "approve",
            "--reason",
            "looks good",
            "--replay",
            replay_id,
        ],
    );
    assert!(
        out.status.success(),
        "verdict failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Verdict should be in .musketeer/verdicts/
    let verdict_path = tmp
        .path()
        .join(format!(".musketeer/verdicts/{}.verdict.yml", replay_id));
    assert!(verdict_path.is_file(), "verdict file should exist");

    // No legacy artifacts
    let musketeer_dir = tmp.path().join(".musketeer");
    let legacy = find_legacy_artifacts(&musketeer_dir);
    assert!(
        legacy.is_empty(),
        "Legacy artifacts found after verdict in SMALL-native mode: {:?}",
        legacy
    );
}

#[test]
fn check_creates_no_legacy_artifacts() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-check-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(tmp.path(), &["check", "--replay", replay_id]);
    assert!(
        out.status.success(),
        "check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let musketeer_dir = tmp.path().join(".musketeer");
    let legacy = find_legacy_artifacts(&musketeer_dir);
    assert!(
        legacy.is_empty(),
        "Legacy artifacts found after check in SMALL-native mode: {:?}",
        legacy
    );
}

#[test]
fn packet_creates_no_legacy_artifacts() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-packet-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(
        tmp.path(),
        &[
            "--json", "packet", "--role", "executor", "--replay", replay_id,
        ],
    );
    assert!(
        out.status.success(),
        "packet failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let musketeer_dir = tmp.path().join(".musketeer");
    let legacy = find_legacy_artifacts(&musketeer_dir);
    assert!(
        legacy.is_empty(),
        "Legacy artifacts found after packet in SMALL-native mode: {:?}",
        legacy
    );
}

#[test]
fn full_workflow_no_legacy_artifacts() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-full-001";
    setup_small_workspace(tmp.path(), replay_id);

    // run new
    let out = run_cmd(tmp.path(), &["--json", "run", "new"]);
    assert!(
        out.status.success(),
        "run new failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // log
    let out = run_cmd(
        tmp.path(),
        &[
            "--json",
            "log",
            "--role",
            "executor",
            "--kind",
            "note",
            "--message",
            "doing work",
            "--replay",
            replay_id,
        ],
    );
    assert!(
        out.status.success(),
        "log failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // packet
    let out = run_cmd(
        tmp.path(),
        &[
            "--json", "packet", "--role", "executor", "--replay", replay_id,
        ],
    );
    assert!(
        out.status.success(),
        "packet failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // verdict
    let out = run_cmd(
        tmp.path(),
        &[
            "--json", "verdict", "--role", "auditor", "--value", "approve", "--reason", "all good",
            "--replay", replay_id,
        ],
    );
    assert!(
        out.status.success(),
        "verdict failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // check
    let out = run_cmd(tmp.path(), &["check", "--replay", replay_id]);
    assert!(
        out.status.success(),
        "check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Final scan: NO legacy artifacts anywhere under .musketeer/
    let musketeer_dir = tmp.path().join(".musketeer");
    let legacy = find_legacy_artifacts(&musketeer_dir);
    assert!(
        legacy.is_empty(),
        "Legacy artifacts found after full workflow in SMALL-native mode: {:?}",
        legacy
    );

    // Verify execution log was written
    // run new creates a UUID run dir, but log uses the replay_id
    let exec_log = tmp
        .path()
        .join(format!(".musketeer/runs/{}/execution-log.yml", replay_id));
    assert!(
        exec_log.is_file(),
        "execution-log.yml should exist after log command"
    );
}

#[test]
fn execution_log_schema_is_correct() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "p4-schema-001";
    setup_small_workspace(tmp.path(), replay_id);
    fs::create_dir_all(tmp.path().join(format!(".musketeer/runs/{}", replay_id))).unwrap();

    // Log two entries
    for msg in &["first entry", "second entry"] {
        let out = run_cmd(
            tmp.path(),
            &[
                "--json",
                "log",
                "--role",
                "executor",
                "--kind",
                "note",
                "--message",
                msg,
                "--replay",
                replay_id,
            ],
        );
        assert!(
            out.status.success(),
            "log failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let exec_log_path = tmp
        .path()
        .join(format!(".musketeer/runs/{}/execution-log.yml", replay_id));
    let content = fs::read_to_string(&exec_log_path).unwrap();

    assert!(content.contains("kind: musketeer_execution_log"));
    assert!(content.contains("musketeer_version:"));
    assert!(content.contains(&format!("replay_id: {}", replay_id)));
    assert!(content.contains("first entry"));
    assert!(content.contains("second entry"));
    assert!(content.contains("seq: 1"));
    assert!(content.contains("seq: 2"));
}
