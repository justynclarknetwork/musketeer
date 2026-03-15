//! Phase 3 integration tests: read-path convergence for SMALL-native workspaces.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn musketeer_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps
    path.pop(); // debug
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
        "title: SMALL Intent\noutcome: SMALL Outcome\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("constraints.small.yml"),
        "scope:\n  - src/\nnon_goals:\n  - vendor/\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("plan.small.yml"),
        "tasks:\n  - id: s1\n    title: Small Task\n    status: pending\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("progress.small.yml"),
        "entries:\n  - seq: 1\n    ts: '2026-01-01T00:00:00Z'\n    role: executor\n    kind: note\n    message: init\n    summary: init\n",
    )
    .unwrap();
    fs::write(
        small_dir.join("handoff.small.yml"),
        "note: Ready\n",
    )
    .unwrap();
    // Also create .musketeer/ for Musketeer execution state
    fs::create_dir_all(root.join(".musketeer/runs")).unwrap();
}

fn setup_legacy_workspace(root: &Path) {
    fs::create_dir_all(root.join(".musketeer")).unwrap();
    run_cmd(root, &["init"]);
}

#[test]
fn packet_reads_from_small_workspace() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "small-replay-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(
        tmp.path(),
        &["--json", "packet", "--role", "executor", "--replay", replay_id],
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "packet failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(stdout.contains("SMALL Intent"), "expected SMALL Intent in output: {}", stdout);
    assert!(stdout.contains("SMALL Outcome"), "expected SMALL Outcome in output: {}", stdout);
}

#[test]
fn check_validates_small_artifacts() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "small-check-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(tmp.path(), &["check", "--replay", replay_id]);
    assert!(out.status.success(), "check failed: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn check_fails_with_missing_small_artifact() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "small-missing-001";
    setup_small_workspace(tmp.path(), replay_id);

    fs::remove_file(tmp.path().join(".small/plan.small.yml")).unwrap();

    let out = run_cmd(tmp.path(), &["check", "--replay", replay_id]);
    assert!(!out.status.success(), "check should fail with missing artifact");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("missing"), "expected 'missing' in error: {}", stderr);
}

#[test]
fn run_new_in_small_mode_does_not_create_artifacts() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "small-new-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(tmp.path(), &["--json", "run", "new"]);
    assert!(out.status.success(), "run new failed: {}", String::from_utf8_lossy(&out.stderr));

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("small_native"), "expected small_native mode: {}", stdout);

    // Verify NO legacy artifacts were created
    let runs_dir = tmp.path().join(".musketeer/runs");
    if runs_dir.exists() {
        for entry in fs::read_dir(&runs_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                assert!(
                    !entry.path().join("intent.yml").exists(),
                    "SMALL-native run new should NOT create legacy intent.yml"
                );
            }
        }
    }
}

#[test]
fn legacy_fallback_emits_deprecation_warning() {
    let tmp = TempDir::new().unwrap();
    setup_legacy_workspace(tmp.path());

    let out = run_cmd(tmp.path(), &["run", "new"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("deprecated"),
        "expected deprecation warning: {}",
        stderr
    );
}

#[test]
fn replay_id_conflict_fails() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "small-conflict-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(
        tmp.path(),
        &["packet", "--role", "executor", "--replay", "wrong-id"],
    );
    assert!(!out.status.success(), "should fail with conflicting replay ID");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("conflict"),
        "expected conflict error: {}",
        stderr
    );
}

#[test]
fn run_status_small_native_shows_plan() {
    let tmp = TempDir::new().unwrap();
    let replay_id = "small-status-001";
    setup_small_workspace(tmp.path(), replay_id);

    let out = run_cmd(tmp.path(), &["--json", "run", "status"]);
    assert!(out.status.success(), "run status failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("small_native"), "expected small_native: {}", stdout);
}
