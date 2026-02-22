use std::env;

use anyhow::Context;

use crate::commands::util;
use crate::error::MusketeerError;
use crate::fs::{layout, read};
use crate::invariants::check::check_run;
use crate::model::run::Handoff;
use crate::output;

pub fn run(replay: Option<String>, json_mode: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let replay_id = match replay {
        Some(id) => id,
        None => util::latest_replay_id(&root)?,
    };

    let result = check_run(&root, &replay_id);
    if !result.ok {
        return Err(MusketeerError::InvariantFailed(result.errors.join("; ")).into());
    }

    let handoff: Handoff = read::read_yaml(&layout::handoff_path(&root, &replay_id))?;
    if handoff.verdict.as_deref() == Some("reject") {
        return Err(MusketeerError::VerdictRejected(
            handoff
                .verdict_reason
                .unwrap_or_else(|| "auditor rejected".to_string()),
        )
        .into());
    }

    if json_mode {
        output::emit_ok(json_mode, Some(&replay_id), serde_json::json!({}));
    } else {
        println!("check ok: {replay_id}");
    }
    Ok(())
}
