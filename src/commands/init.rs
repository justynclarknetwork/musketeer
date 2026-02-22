use std::env;

use anyhow::Context;

use crate::fs::{layout, write};
use crate::model::config::{AgentSeat, Agents, Config, Policy, Redaction, Workspace};
use crate::output;

pub fn run(json_mode: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let state = layout::state_dir(&root);
    let runs = layout::runs_dir(&root);
    write::ensure_dir(&state)?;
    write::ensure_dir(&runs)?;

    let cfg = Config {
        version: 1,
        workspace: Workspace {
            state_dir: ".musketeer".to_string(),
        },
        agents: Agents {
            originator: AgentSeat {
                adapter: "manual".to_string(),
            },
            cross_examiner: AgentSeat {
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
    if json_mode {
        output::emit_ok(json_mode, None, serde_json::json!({}));
    } else {
        println!("workspace ready in {}", state.display());
    }
    Ok(())
}
