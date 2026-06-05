//! Phase 5 system evidence test.
//!
//! This is a concrete end-to-end use case that exercises the fresh SMALL-native
//! path and scores whether the system produces coherent, usable outputs.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const FIXED_REPLAY_ID: &str = "evidence-use-case-001";
const USE_CASE_TITLE: &str = "Ship a clean SMALL-native first run";
const USE_CASE_OUTCOME: &str = "Fresh users can initialize, inspect packets, log evidence, and close a run without legacy noise";

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

fn assert_clean_stderr(label: &str, stderr: &[u8]) {
    let text = String::from_utf8_lossy(stderr);
    assert!(
        !text.to_ascii_lowercase().contains("deprecated"),
        "{label} emitted deprecation noise: {text}"
    );
}

fn write_use_case_small_workspace(root: &Path) {
    fs::create_dir_all(root.join(".small")).unwrap();
    fs::write(
        root.join(".small/workspace.small.yml"),
        format!("version: 1\nreplay_id: {}\n", FIXED_REPLAY_ID),
    )
    .unwrap();
    fs::write(
        root.join(".small/intent.small.yml"),
        format!("title: {}\noutcome: {}\n", USE_CASE_TITLE, USE_CASE_OUTCOME),
    )
    .unwrap();
    fs::write(
        root.join(".small/constraints.small.yml"),
        "scope:\n  - README.md\n  - DEMO.md\n  - scripts/feeltest.sh\nnon_goals:\n  - add network services\n  - change bridge protocol\n",
    )
    .unwrap();
    fs::write(
        root.join(".small/plan.small.yml"),
        "tasks:\n  - id: t1\n    title: verify fresh init bootstraps canonical SMALL artifacts\n    status: done\n  - id: t2\n    title: inspect planner packet for the real use case\n    status: pending\n  - id: t3\n    title: capture executor evidence and close with auditor verdict\n    status: pending\n",
    )
    .unwrap();
    fs::write(
        root.join(".small/progress.small.yml"),
        "entries:\n  - seq: 1\n    ts: '2026-04-23T21:00:00Z'\n    role: planner\n    kind: note\n    message: acceptance evidence criteria defined\n    summary: acceptance evidence criteria defined\n  - seq: 2\n    ts: '2026-04-23T21:05:00Z'\n    role: executor\n    kind: evidence\n    message: baseline warning path reproduced before hardening\n    summary: baseline warning path reproduced before hardening\n",
    )
    .unwrap();
    fs::write(
        root.join(".small/handoff.small.yml"),
        "note: Ready for role-separated packet review and execution receipt capture\n",
    )
    .unwrap();
}

fn legacy_artifacts(root: &Path) -> Vec<String> {
    let legacy_names = [
        "intent.yml",
        "constraints.yml",
        "plan.yml",
        "progress.yml",
        "handoff.yml",
    ];
    let mut found = Vec::new();
    let base = root.join(".musketeer");
    if !base.exists() {
        return found;
    }
    walk_files(&base, &mut |path| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if legacy_names.contains(&name) {
                found.push(path.display().to_string());
            }
        }
    });
    found
}

fn walk_files(dir: &Path, f: &mut impl FnMut(&Path)) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_files(&path, f);
            } else {
                f(&path);
            }
        }
    }
}

#[test]
fn concrete_use_case_produces_evidence_of_coherency_and_usability() {
    let tmp = TempDir::new().unwrap();

    let init = run_cmd(tmp.path(), &["init", "--json"]);
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert_clean_stderr("init", &init.stderr);
    let init_json: serde_json::Value = serde_json::from_slice(&init.stdout).unwrap();
    assert_eq!(init_json["tool"], "musketeer");
    assert_eq!(init_json["status"], "ok");
    assert_eq!(init_json["mode"], "small_native");

    write_use_case_small_workspace(tmp.path());

    let run_new = run_cmd(tmp.path(), &["run", "new", "--json"]);
    assert!(
        run_new.status.success(),
        "run new failed: {}",
        String::from_utf8_lossy(&run_new.stderr)
    );
    assert_clean_stderr("run new", &run_new.stderr);
    let run_new_json: serde_json::Value = serde_json::from_slice(&run_new.stdout).unwrap();
    assert_eq!(run_new_json["replay_id"], FIXED_REPLAY_ID);
    assert_eq!(run_new_json["mode"], "small_native");

    let planner_packet = run_cmd(
        tmp.path(),
        &[
            "packet",
            "--role",
            "planner",
            "--replay",
            FIXED_REPLAY_ID,
            "--json",
        ],
    );
    assert!(
        planner_packet.status.success(),
        "planner packet failed: {}",
        String::from_utf8_lossy(&planner_packet.stderr)
    );
    assert_clean_stderr("planner packet", &planner_packet.stderr);
    let planner_json: serde_json::Value = serde_json::from_slice(&planner_packet.stdout).unwrap();

    let executor_packet = run_cmd(
        tmp.path(),
        &[
            "packet",
            "--role",
            "executor",
            "--replay",
            FIXED_REPLAY_ID,
            "--json",
        ],
    );
    assert!(
        executor_packet.status.success(),
        "executor packet failed: {}",
        String::from_utf8_lossy(&executor_packet.stderr)
    );
    assert_clean_stderr("executor packet", &executor_packet.stderr);
    let executor_json: serde_json::Value = serde_json::from_slice(&executor_packet.stdout).unwrap();

    let auditor_packet = run_cmd(
        tmp.path(),
        &[
            "packet",
            "--role",
            "auditor",
            "--replay",
            FIXED_REPLAY_ID,
            "--json",
        ],
    );
    assert!(
        auditor_packet.status.success(),
        "auditor packet failed: {}",
        String::from_utf8_lossy(&auditor_packet.stderr)
    );
    assert_clean_stderr("auditor packet", &auditor_packet.stderr);
    let auditor_json: serde_json::Value = serde_json::from_slice(&auditor_packet.stdout).unwrap();

    let log_decision = run_cmd(
        tmp.path(),
        &[
            "log",
            "--role",
            "planner",
            "--kind",
            "decision",
            "--message",
            "fresh init must own canonical SMALL bootstrap",
            "--replay",
            FIXED_REPLAY_ID,
            "--json",
        ],
    );
    assert!(
        log_decision.status.success(),
        "decision log failed: {}",
        String::from_utf8_lossy(&log_decision.stderr)
    );
    assert_clean_stderr("decision log", &log_decision.stderr);
    let log_decision_json: serde_json::Value =
        serde_json::from_slice(&log_decision.stdout).unwrap();
    assert_eq!(log_decision_json["seq"], 1);
    assert_eq!(log_decision_json["kind"], "decision");

    let log_evidence = run_cmd(
        tmp.path(),
        &[
            "log",
            "--role",
            "executor",
            "--kind",
            "evidence",
            "--message",
            "fresh path completed with stable json and clean stderr",
            "--replay",
            FIXED_REPLAY_ID,
            "--json",
        ],
    );
    assert!(
        log_evidence.status.success(),
        "evidence log failed: {}",
        String::from_utf8_lossy(&log_evidence.stderr)
    );
    assert_clean_stderr("evidence log", &log_evidence.stderr);
    let log_evidence_json: serde_json::Value =
        serde_json::from_slice(&log_evidence.stdout).unwrap();
    assert_eq!(log_evidence_json["seq"], 2);
    assert_eq!(log_evidence_json["kind"], "evidence");

    let verdict = run_cmd(
        tmp.path(),
        &[
            "verdict",
            "--role",
            "auditor",
            "--value",
            "approve",
            "--reason",
            "coherent SMALL-native first run with durable evidence",
            "--replay",
            FIXED_REPLAY_ID,
            "--json",
        ],
    );
    assert!(
        verdict.status.success(),
        "verdict failed: {}",
        String::from_utf8_lossy(&verdict.stderr)
    );
    assert_clean_stderr("verdict", &verdict.stderr);
    let verdict_json: serde_json::Value = serde_json::from_slice(&verdict.stdout).unwrap();
    assert_eq!(verdict_json["verdict"], "approve");

    let check = run_cmd(
        tmp.path(),
        &["check", "--replay", FIXED_REPLAY_ID, "--json"],
    );
    assert!(
        check.status.success(),
        "check failed: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert_clean_stderr("check", &check.stderr);
    let check_json: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(check_json["status"], "ok");

    let run_status = run_cmd(
        tmp.path(),
        &["run", "status", "--replay", FIXED_REPLAY_ID, "--json"],
    );
    assert!(
        run_status.status.success(),
        "run status failed: {}",
        String::from_utf8_lossy(&run_status.stderr)
    );
    assert_clean_stderr("run status", &run_status.stderr);
    let run_status_json: serde_json::Value = serde_json::from_slice(&run_status.stdout).unwrap();
    assert_eq!(run_status_json["mode"], "small_native");
    assert_eq!(run_status_json["done"], 1);
    assert_eq!(run_status_json["total"], 3);

    assert_eq!(planner_json["intent"]["title"], USE_CASE_TITLE);
    assert_eq!(planner_json["intent"]["outcome"], USE_CASE_OUTCOME);
    assert_eq!(
        planner_json["constraints"]["scope"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(planner_json["plan_slice"].as_array().unwrap().len(), 2);
    assert_eq!(planner_json["progress_slice"].as_array().unwrap().len(), 2);
    assert_eq!(planner_json["next_expected_action"], "execute_next_task");
    assert_eq!(planner_json["role"], "planner");
    assert_eq!(executor_json["role"], "executor");
    assert_eq!(auditor_json["role"], "auditor");
    assert_eq!(executor_json["plan_slice"], planner_json["plan_slice"]);
    assert_eq!(auditor_json["plan_slice"], planner_json["plan_slice"]);
    assert_eq!(
        executor_json["progress_slice"],
        planner_json["progress_slice"]
    );
    assert_eq!(
        auditor_json["progress_slice"],
        planner_json["progress_slice"]
    );

    let execution_log_path = tmp.path().join(format!(
        ".musketeer/runs/{}/execution-log.yml",
        FIXED_REPLAY_ID
    ));
    assert!(execution_log_path.is_file(), "execution log missing");
    let execution_log: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&execution_log_path).unwrap()).unwrap();
    let entries = execution_log["entries"].as_sequence().unwrap();
    assert_eq!(
        execution_log["kind"].as_str(),
        Some("musketeer_execution_log")
    );
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["seq"].as_u64(), Some(1));
    assert_eq!(entries[1]["seq"].as_u64(), Some(2));
    assert_eq!(entries[0]["kind"].as_str(), Some("decision"));
    assert_eq!(entries[1]["kind"].as_str(), Some("evidence"));

    let verdict_path = tmp.path().join(format!(
        ".musketeer/verdicts/{}.verdict.yml",
        FIXED_REPLAY_ID
    ));
    assert!(verdict_path.is_file(), "verdict file missing");

    let legacy = legacy_artifacts(tmp.path());
    assert!(
        legacy.is_empty(),
        "legacy artifacts leaked into SMALL-native run: {:?}",
        legacy
    );

    let proof_checks = [
        init_json["mode"] == "small_native",
        run_new_json["replay_id"] == FIXED_REPLAY_ID,
        check_json["status"] == "ok",
        legacy.is_empty(),
    ];
    let coherency_checks = [
        planner_json["intent"]["title"] == USE_CASE_TITLE,
        planner_json["next_expected_action"] == "execute_next_task",
        executor_json["plan_slice"] == planner_json["plan_slice"],
        entries[0]["seq"].as_u64() == Some(1) && entries[1]["seq"].as_u64() == Some(2),
    ];
    let usability_checks = [
        init_json["tool"] == "musketeer",
        run_status_json["mode"] == "small_native",
        log_evidence_json["kind"] == "evidence",
        verdict_json["verdict"] == "approve",
    ];

    let proof_score = proof_checks.iter().filter(|ok| **ok).count();
    let coherency_score = coherency_checks.iter().filter(|ok| **ok).count();
    let usability_score = usability_checks.iter().filter(|ok| **ok).count();

    assert_eq!(proof_score, proof_checks.len(), "proof score degraded");
    assert_eq!(
        coherency_score,
        coherency_checks.len(),
        "coherency score degraded"
    );
    assert_eq!(
        usability_score,
        usability_checks.len(),
        "usability score degraded"
    );
}
