use std::env;

use anyhow::Context;

use crate::commands::util;
use crate::error::MusketeerError;
use crate::fs::{layout, read, write};
use crate::model::run::Handoff;
use crate::output;

pub fn run(
    role: String,
    value: String,
    reason: String,
    replay: Option<String>,
    json_mode: bool,
) -> anyhow::Result<()> {
    if role != "auditor" {
        return Err(
            MusketeerError::RoleViolation("only auditor can set verdict".to_string()).into(),
        );
    }
    if value != "approve" && value != "reject" {
        return Err(
            MusketeerError::InvalidInput("verdict must be approve|reject".to_string()).into(),
        );
    }
    if reason.trim().is_empty() {
        return Err(MusketeerError::InvalidInput("reason is empty".to_string()).into());
    }

    let root = env::current_dir().context("failed to resolve current dir")?;
    let replay_id = replay.unwrap_or(util::latest_replay_id(&root)?);
    let handoff_path = layout::handoff_path(&root, &replay_id);
    let mut handoff: Handoff = read::read_yaml(&handoff_path)
        .map_err(|_| MusketeerError::HandoffInvalid("handoff missing".to_string()))?;

    if value == "approve" {
        handoff.verdict = Some("approve".to_string());
        handoff.verdict_reason = Some(reason);
    } else {
        handoff.verdict = Some("reject".to_string());
        handoff.verdict_reason = Some(reason);
    }

    write::write_yaml(&handoff_path, &handoff)?;

    if json_mode {
        output::emit_ok(
            json_mode,
            Some(&replay_id),
            serde_json::json!({"verdict": value}),
        );
    } else {
        println!("verdict recorded");
    }
    Ok(())
}
