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
