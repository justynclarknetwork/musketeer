use std::env;

use anyhow::Context;
use serde::Serialize;

use crate::fs::{layout, write};
use crate::model::config::{AgentSeat, Agents, Config, Policy, Redaction, Workspace};
use crate::output;
use crate::small_workspace;

#[derive(Serialize)]
struct SmallWorkspaceBootstrap {
    version: u8,
}

#[derive(Serialize)]
struct SmallIntentBootstrap {
    title: String,
    outcome: String,
}

#[derive(Serialize)]
struct SmallConstraintsBootstrap {
    scope: Vec<String>,
    non_goals: Vec<String>,
}

#[derive(Serialize)]
struct SmallPlanBootstrap {
    tasks: Vec<serde_yaml::Value>,
}

#[derive(Serialize)]
struct SmallProgressBootstrap {
    entries: Vec<serde_yaml::Value>,
}

#[derive(Serialize)]
struct SmallHandoffBootstrap {
    note: String,
}

pub fn run(json_mode: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let state = layout::state_dir(&root);
    let runs = layout::runs_dir(&root);
    let small_dir = small_workspace::small_dir(&root);
    write::ensure_dir(&state)?;
    write::ensure_dir(&runs)?;
    write::ensure_dir(&small_dir)?;

    let cfg = Config {
        version: 1,
        workspace: Workspace {
            state_dir: ".musketeer".to_string(),
        },
        agents: Agents {
            originator: AgentSeat {
                adapter: "manual".to_string(),
            },
            examiner: AgentSeat {
                adapter: "manual".to_string(),
            },
            executor: AgentSeat {
                adapter: "manual".to_string(),
            },
        },
        policy: Policy {
            executor_allowlist: vec!["cargo".to_string(), "git".to_string()],
            redaction: Redaction {
                enabled: false,
                patterns: Vec::new(),
            },
        },
    };

    write::write_yaml(&layout::config_path(&root), &cfg)?;
    write_if_missing(
        &small_workspace::workspace_path(&root),
        &SmallWorkspaceBootstrap { version: 1 },
    )?;
    write_if_missing(
        &small_workspace::intent_path(&root),
        &SmallIntentBootstrap {
            title: "Untitled".to_string(),
            outcome: "TBD".to_string(),
        },
    )?;
    write_if_missing(
        &small_workspace::constraints_path(&root),
        &SmallConstraintsBootstrap {
            scope: Vec::new(),
            non_goals: Vec::new(),
        },
    )?;
    write_if_missing(
        &small_workspace::plan_path(&root),
        &SmallPlanBootstrap { tasks: Vec::new() },
    )?;
    write_if_missing(
        &small_workspace::progress_path(&root),
        &SmallProgressBootstrap {
            entries: Vec::new(),
        },
    )?;
    write_if_missing(
        &small_workspace::handoff_path(&root),
        &SmallHandoffBootstrap {
            note: "".to_string(),
        },
    )?;

    if json_mode {
        output::emit_ok(json_mode, None, serde_json::json!({"mode": "small_native"}));
    } else {
        println!("workspace ready in {}", state.display());
    }
    Ok(())
}

fn write_if_missing<T: Serialize>(path: &std::path::Path, value: &T) -> anyhow::Result<()> {
    if !path.exists() {
        write::write_yaml(path, value)?;
    }
    Ok(())
}
