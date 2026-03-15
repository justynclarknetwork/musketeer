use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_musketeer").to_string()
}

fn run(args: &[&str], cwd: &std::path::Path) -> (i32, String, String) {
    let out = Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run cli");
    (
        out.status.code().unwrap_or(50),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn setup(cwd: &std::path::Path) -> String {
    let (c, out, _) = run(&["init", "--json"], cwd);
    assert_eq!(c, 0, "init failed");
    let (c, out2, _) = run(&["run", "new", "--json"], cwd);
    assert_eq!(c, 0, "run new failed");
    drop(out);
    let v: serde_json::Value = serde_json::from_str(&out2).expect("json");
    v["replay_id"].as_str().expect("replay_id").to_string()
}

// --- init ---

#[test]
fn init_creates_workspace_files() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    let (c, out, _) = run(&["init", "--json"], cwd);
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["tool"], "musketeer");
    assert_eq!(v["status"], "ok");
    assert!(cwd.join(".musketeer").exists(), ".musketeer dir missing");
    assert!(
        cwd.join(".musketeer/musketeer.yml").exists(),
        "musketeer.yml missing"
    );
    assert!(cwd.join(".musketeer/runs").exists(), "runs dir missing");
}

#[test]
fn init_is_idempotent() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    let (c, _, _) = run(&["init", "--json"], cwd);
    assert_eq!(c, 0);
    let (c, out, _) = run(&["init", "--json"], cwd);
    assert_eq!(c, 0, "second init failed");
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["status"], "ok");
}

// --- run new ---

#[test]
fn run_new_creates_artifacts() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let run_dir = cwd.join(".musketeer/runs").join(&replay_id);
    assert!(run_dir.exists(), "run dir missing");
    for f in &[
        "intent.yml",
        "constraints.yml",
        "plan.yml",
        "progress.yml",
        "handoff.yml",
    ] {
        assert!(run_dir.join(f).exists(), "{f} missing");
    }
}

#[test]
fn run_new_fails_without_workspace() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    let (c, out, _) = run(&["run", "new", "--json"], cwd);
    assert_eq!(c, 30);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["status"], "error");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_WORKSPACE_INVALID"));
}

// --- run status ---

#[test]
fn run_status_shows_run() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(&["run", "status", "--replay", &replay_id, "--json"], cwd);
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["done"], 0);
    assert_eq!(v["total"], 0);
}

#[test]
fn run_status_fails_no_workspace() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    let (c, out, _) = run(&["run", "status", "--json"], cwd);
    assert_eq!(c, 30);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["status"], "error");
}

// --- check ---

#[test]
fn check_passes_fresh_run() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(&["check", "--replay", &replay_id, "--json"], cwd);
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["status"], "ok");
}

#[test]
fn check_fails_no_workspace() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    let (c, out, _) = run(&["check", "--json"], cwd);
    assert_eq!(c, 30);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_WORKSPACE_INVALID"));
}

// --- packet ---

#[test]
fn packet_returns_role_and_intent() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "packet", "--role", "executor", "--replay", &replay_id, "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["role"], "executor");
    assert!(v["intent"].is_object(), "intent missing");
    assert!(v["plan_slice"].is_array(), "plan_slice missing");
    assert!(v["progress_slice"].is_array(), "progress_slice missing");
}

#[test]
fn packet_fails_invalid_role() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "packet",
            "--role",
            "not-a-role",
            "--replay",
            &replay_id,
            "--json",
        ],
        cwd,
    );
    assert_eq!(c, 21);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_ROLE_VIOLATION"));
}

// --- log ---

#[test]
fn log_appends_entry_with_seq() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "log",
            "--role",
            "executor",
            "--kind",
            "note",
            "--message",
            "step done",
            "--replay",
            &replay_id,
            "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["seq"], 1);
    assert_eq!(v["kind"], "note");
}

#[test]
fn log_fails_invalid_kind() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "log",
            "--role",
            "executor",
            "--kind",
            "bad-kind",
            "--message",
            "x",
            "--replay",
            &replay_id,
            "--json",
        ],
        cwd,
    );
    assert_eq!(c, 40);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_INVALID_INPUT"));
}

#[test]
fn log_fails_invalid_role() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "log",
            "--role",
            "bad-role",
            "--kind",
            "note",
            "--message",
            "x",
            "--replay",
            &replay_id,
            "--json",
        ],
        cwd,
    );
    assert_eq!(c, 21);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_ROLE_VIOLATION"));
}

// --- verdict ---

#[test]
fn verdict_records_approve() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "verdict",
            "--role",
            "auditor",
            "--value",
            "approve",
            "--reason",
            "looks good",
            "--replay",
            &replay_id,
            "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["verdict"], "approve");
}

#[test]
fn verdict_fails_non_auditor() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "verdict", "--role", "executor", "--value", "approve", "--reason", "x", "--replay",
            &replay_id, "--json",
        ],
        cwd,
    );
    assert_eq!(c, 21);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_ROLE_VIOLATION"));
}

#[test]
fn verdict_fails_invalid_value() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();
    let replay_id = setup(cwd);

    let (c, out, _) = run(
        &[
            "verdict", "--role", "auditor", "--value", "maybe", "--reason", "x", "--replay",
            &replay_id, "--json",
        ],
        cwd,
    );
    assert_eq!(c, 40);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_INVALID_INPUT"));
}

#[test]
fn verdict_fails_no_run_exists() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    run(&["init", "--json"], cwd);
    // No run new - no runs exist
    let (c, out, _) = run(
        &[
            "verdict", "--role", "auditor", "--value", "approve", "--reason", "x", "--json",
        ],
        cwd,
    );
    assert_eq!(c, 30);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "E_WORKSPACE_INVALID"));
}

// --- full workflow ---

#[test]
fn json_contract_and_exit_codes() {
    let td = tempfile::tempdir().expect("tempdir");
    let cwd = td.path();

    let (c, out, _) = run(&["init", "--json"], cwd);
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["tool"], "musketeer");
    assert!(v["errors"].is_array());

    let (c, out, _) = run(&["run", "new", "--json"], cwd);
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    let replay_id = v["replay_id"].as_str().expect("replay_id").to_string();

    let (c, out, _) = run(
        &[
            "packet", "--role", "planner", "--replay", &replay_id, "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["role"], "planner");

    let (c, out, _) = run(
        &[
            "log",
            "--role",
            "executor",
            "--kind",
            "note",
            "--message",
            "x",
            "--replay",
            &replay_id,
            "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["seq"], 1);

    let (c, _, _) = run(
        &[
            "verdict", "--role", "auditor", "--value", "reject", "--reason", "x", "--replay",
            &replay_id, "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);

    let (c, out, _) = run(&["check", "--replay", &replay_id, "--json"], cwd);
    assert_eq!(c, 23);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(v["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .any(|e| e == "E_VERDICT_REJECTED"));

    let (c, _, _) = run(
        &[
            "verdict", "--role", "auditor", "--value", "approve", "--reason", "x", "--replay",
            &replay_id, "--json",
        ],
        cwd,
    );
    assert_eq!(c, 0);

    let (c, out, _) = run(&["check", "--replay", &replay_id, "--json"], cwd);
    assert_eq!(c, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["status"], "ok");
}
