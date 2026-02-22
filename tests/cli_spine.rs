use std::env;
use std::fs;

use std::sync::{Mutex, OnceLock};

use tempfile::TempDir;

use musketeer::fs::{layout, read, write};
use musketeer::invariants::check::check_run;
use musketeer::model::progress::{ProgressEntry, ProgressLog};
use musketeer::model::run::Intent;

fn setup_temp_workspace() -> TempDir {
    let temp_dir = TempDir::new().expect("temp dir");
    env::set_current_dir(temp_dir.path()).expect("set current dir");
    temp_dir
}

fn init_workspace() {
    musketeer::commands::init::run(false).expect("init workspace");
}

fn create_run() -> String {
    musketeer::commands::run_new::run(false).expect("run new");
    let runs_dir = layout::runs_dir(&env::current_dir().unwrap());
    let mut entries: Vec<String> = fs::read_dir(runs_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
        .collect();
    entries.sort();
    entries.pop().expect("replay id")
}

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn init_creates_state_dir() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();

    let root = env::current_dir().unwrap();
    assert!(layout::state_dir(&root).exists());
    assert!(layout::config_path(&root).exists());
    assert!(layout::runs_dir(&root).exists());
}

#[test]
fn run_new_creates_run_files() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    assert!(layout::run_dir(&root, &replay_id).exists());
    assert!(layout::intent_path(&root, &replay_id).exists());
    assert!(layout::constraints_path(&root, &replay_id).exists());
    assert!(layout::plan_path(&root, &replay_id).exists());
    assert!(layout::progress_path(&root, &replay_id).exists());
    assert!(layout::handoff_path(&root, &replay_id).exists());
}

#[test]
fn check_passes_on_fresh_run() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    let result = check_run(&root, &replay_id);
    assert!(result.ok, "expected ok, got: {:?}", result.errors);
}

#[test]
fn check_fails_if_missing_file() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    fs::remove_file(layout::intent_path(&root, &replay_id)).unwrap();
    let result = check_run(&root, &replay_id);
    assert!(!result.ok);
    assert!(result
        .errors
        .iter()
        .any(|err| err.contains("missing required file intent")));
}

#[test]
fn check_fails_if_replay_id_mismatch() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    let mut intent: Intent = read::read_yaml(&layout::intent_path(&root, &replay_id)).unwrap();
    intent.replay_id = "wrong".to_string();
    write::write_yaml(&layout::intent_path(&root, &replay_id), &intent).unwrap();

    let result = check_run(&root, &replay_id);
    assert!(!result.ok);
    assert!(result
        .errors
        .iter()
        .any(|err| err.contains("replay_id mismatch")));
}

#[test]
fn check_fails_if_progress_seq_not_increasing() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    let progress = ProgressLog {
        replay_id: replay_id.clone(),
        entries: vec![
            ProgressEntry {
                seq: 1,
                ts: "2024-01-01T00:00:00Z".to_string(),
                role: "planner".to_string(),
                kind: "note".to_string(),
                message: "first".to_string(),
                summary: "first".to_string(),
            },
            ProgressEntry {
                seq: 1,
                ts: "2024-01-01T00:00:01Z".to_string(),
                role: "executor".to_string(),
                kind: "note".to_string(),
                message: "second".to_string(),
                summary: "second".to_string(),
            },
        ],
    };
    write::write_yaml(&layout::progress_path(&root, &replay_id), &progress).unwrap();

    let result = check_run(&root, &replay_id);
    assert!(!result.ok);
    assert!(result
        .errors
        .iter()
        .any(|err| err.contains("strictly increasing")));
}
