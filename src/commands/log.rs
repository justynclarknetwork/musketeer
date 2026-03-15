use std::env;

use anyhow::Context;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::MusketeerError;
use crate::fs::{layout, read, write};
use crate::model::execution_log::{ExecutionEntry, ExecutionLog};
use crate::model::progress::{ProgressEntry, ProgressLog};
use crate::musketeer_namespace;
use crate::output;
use crate::workspace_mode::{self, WorkspaceContext};

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
    let ctx = workspace_mode::resolve(&root)?;
    let replay_id = workspace_mode::resolve_replay_id(&ctx, replay)?;

    let ts = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format timestamp")?;

    match &ctx {
        WorkspaceContext::SmallNative { .. } => {
            // Read existing execution log or create new one
            let log_path = musketeer_namespace::execution_log_path(&root, &replay_id);
            let mut exec_log: ExecutionLog = if log_path.is_file() {
                read::read_yaml(&log_path).unwrap_or_else(|_| ExecutionLog::new(&replay_id))
            } else {
                ExecutionLog::new(&replay_id)
            };

            let next_seq = exec_log.entries.last().map(|e| e.seq + 1).unwrap_or(1);
            exec_log.entries.push(ExecutionEntry {
                seq: next_seq,
                ts,
                role,
                kind: kind.clone(),
                message: message.clone(),
            });

            // Ensure run dir exists and write execution log
            let run_dir = musketeer_namespace::run_dir(&root, &replay_id);
            write::ensure_dir(&run_dir)?;
            write::write_yaml(&log_path, &exec_log)?;

            if json_mode {
                output::emit_ok(
                    json_mode,
                    Some(&replay_id),
                    serde_json::json!({"seq": next_seq, "kind": kind}),
                );
            } else {
                println!("logged entry {next_seq}");
            }
        }
        WorkspaceContext::Legacy { .. } => {
            workspace_mode::warn_legacy();
            eprintln!("[deprecated] Legacy workspace detected. SMALL-native mode is preferred. Migration required.");

            // Read existing progress
            let path = layout::progress_path(&root, &replay_id);
            let mut progress: ProgressLog = read::read_yaml(&path)
                .map_err(|_| MusketeerError::HandoffInvalid("progress missing".to_string()))?;

            let next_seq = progress.entries.last().map(|e| e.seq + 1).unwrap_or(1);
            progress.entries.push(ProgressEntry {
                seq: next_seq,
                ts,
                role,
                kind: kind.clone(),
                message: message.clone(),
                summary: message,
            });

            let write_path = layout::progress_path(&root, &replay_id);
            write::write_yaml(&write_path, &progress)?;

            if json_mode {
                output::emit_ok(
                    json_mode,
                    Some(&replay_id),
                    serde_json::json!({"seq": next_seq, "kind": kind}),
                );
            } else {
                println!("logged entry {next_seq}");
            }
        }
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
