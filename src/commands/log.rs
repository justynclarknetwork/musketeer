use std::env;

use anyhow::Context;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::commands::util;
use crate::error::MusketeerError;
use crate::fs::{layout, read, write};
use crate::model::progress::{ProgressEntry, ProgressLog};
use crate::output;

pub fn run(
    role: String,
    kind: String,
    message: String,
    replay: Option<String>,
    json_mode: bool,
) -> anyhow::Result<()> {
    validate_role(&role)?;
    validate_kind(&kind)?;
    if message.trim().is_empty() {
        return Err(MusketeerError::InvalidInput("message is empty".to_string()).into());
    }

    let root = env::current_dir().context("failed to resolve current dir")?;
    let replay_id = replay.unwrap_or(util::latest_replay_id(&root)?);
    let path = layout::progress_path(&root, &replay_id);
    let mut progress: ProgressLog = read::read_yaml(&path)
        .map_err(|_| MusketeerError::HandoffInvalid("progress missing".to_string()))?;

    let next_seq = progress.entries.last().map(|e| e.seq + 1).unwrap_or(1);
    let ts = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format timestamp")?;
    progress.entries.push(ProgressEntry {
        seq: next_seq,
        ts,
        role,
        kind: kind.clone(),
        message: message.clone(),
        summary: message,
    });

    write::write_yaml(&path, &progress)?;

    if json_mode {
        output::emit_ok(
            json_mode,
            Some(&replay_id),
            serde_json::json!({"seq": next_seq, "kind": kind}),
        );
    } else {
        println!("logged entry {next_seq}");
    }

    Ok(())
}

fn validate_role(role: &str) -> anyhow::Result<()> {
    if role == "planner" || role == "executor" || role == "auditor" {
        Ok(())
    } else {
        Err(MusketeerError::RoleViolation(role.to_string()).into())
    }
}

fn validate_kind(kind: &str) -> anyhow::Result<()> {
    if kind == "note" || kind == "decision" || kind == "evidence" {
        Ok(())
    } else {
        Err(MusketeerError::InvalidInput(kind.to_string()).into())
    }
}
