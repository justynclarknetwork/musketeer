use std::env;

use anyhow::Context;

use crate::fs::{layout, write};
use crate::model::config::{AgentSeat, Agents, Config, Policy, Redaction, Workspace};

pub fn run() -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let state_dir = layout::state_dir(&root);
    if state_dir.exists() {
        println!("workspace already initialized");
        return Ok(());
    }

    write::ensure_dir(&layout::runs_dir(&root))?;
    let config = Config {
        version: 1,
        workspace: Workspace {
            state_dir: layout::STATE_DIR.to_string(),
        },
        agents: Agents {
            originator: AgentSeat {
                adapter: "local".to_string(),
            },
            cross_examiner: AgentSeat {
                adapter: "local".to_string(),
            },
            executor: AgentSeat {
                adapter: "local".to_string(),
            },
        },
        policy: Policy {
            executor_allowlist: Vec::new(),
            redaction: Redaction {
                enabled: false,
                patterns: Vec::new(),
            },
        },
    };

    write::write_yaml(&layout::config_path(&root), &config)?;
    write::write_file_atomic(&layout::runs_dir(&root).join(".gitkeep"), b"")?;
    println!("workspace ready in {}", state_dir.display());
    Ok(())
}
