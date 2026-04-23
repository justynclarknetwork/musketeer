use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};

use tempfile::TempDir;

use musketeer::fs::{layout, read};
use musketeer::musketeer_namespace;
use musketeer::small_workspace;

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
fn init_creates_small_and_musketeer_state() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();

    let root = env::current_dir().unwrap();
    assert!(layout::state_dir(&root).exists());
    assert!(layout::config_path(&root).exists());
    assert!(layout::runs_dir(&root).exists());
    assert!(small_workspace::small_dir(&root).exists());
    assert!(small_workspace::workspace_path(&root).exists());
    assert!(small_workspace::intent_path(&root).exists());
    assert!(small_workspace::constraints_path(&root).exists());
    assert!(small_workspace::plan_path(&root).exists());
    assert!(small_workspace::progress_path(&root).exists());
    assert!(small_workspace::handoff_path(&root).exists());
}

#[test]
fn run_new_creates_execution_dir_without_legacy_artifacts() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    let run_dir = layout::run_dir(&root, &replay_id);
    assert!(run_dir.exists());
    assert!(!layout::intent_path(&root, &replay_id).exists());
    assert!(!layout::constraints_path(&root, &replay_id).exists());
    assert!(!layout::plan_path(&root, &replay_id).exists());
    assert!(!layout::progress_path(&root, &replay_id).exists());
    assert!(!layout::handoff_path(&root, &replay_id).exists());
}

#[test]
fn check_passes_on_fresh_small_native_run() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    musketeer::commands::check::run(Some(replay_id), false).expect("check passes");
}

#[test]
fn check_fails_if_missing_small_file() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    let root = env::current_dir().unwrap();
    fs::remove_file(small_workspace::intent_path(&root)).unwrap();
    let err = musketeer::commands::check::run(Some(replay_id), false).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("missing") || msg.contains("intent parse"));
}

#[test]
fn log_writes_execution_log() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    musketeer::commands::log::run(
        "executor".to_string(),
        "note".to_string(),
        "first".to_string(),
        Some(replay_id.clone()),
        false,
    )
    .expect("log entry");

    let root = env::current_dir().unwrap();
    let log_path = musketeer_namespace::execution_log_path(&root, &replay_id);
    assert!(log_path.exists());

    let yaml: serde_yaml::Value = read::read_yaml(&log_path).unwrap();
    let entries = yaml["entries"].as_sequence().unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn verdict_writes_verdict_file() {
    let _guard = test_lock().lock().expect("lock tests");
    let _temp = setup_temp_workspace();
    init_workspace();
    let replay_id = create_run();

    musketeer::commands::verdict::run(
        "auditor".to_string(),
        "approve".to_string(),
        "looks good".to_string(),
        Some(replay_id.clone()),
        false,
    )
    .expect("verdict recorded");

    let root = env::current_dir().unwrap();
    let verdict_path = musketeer_namespace::verdict_path(&root, &replay_id);
    assert!(verdict_path.exists());
}
