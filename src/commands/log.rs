use std::env;

use anyhow::Context;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::MusketeerError;
use crate::fs::{layout, read, write};
use crate::model::progress::{ProgressEntry, ProgressLog};
use crate::output;
use crate::small_adapter;
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

    // Read existing progress
    let mut progress: ProgressLog = match &ctx {
        WorkspaceContext::SmallNative { .. } => {
            small_adapter::read_progress(&root, &replay_id)
                .map_err(|e| MusketeerError::HandoffInvalid(format!("progress: {}", e)))?
        }
        WorkspaceContext::Legacy { .. } => {
            workspace_mode::warn_legacy();
            let path = layout::progress_path(&root, &replay_id);
            read::read_yaml(&path)
                .map_err(|_| MusketeerError::HandoffInvalid("progress missing".to_string()))?
        }
    };

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

    // TODO: In SMALL-native mode, writing progress to the legacy path is a
    // transitional measure. This must move to either:
    //   (a) writing to `.small/progress.small.yml` if SMALL CLI allows external writes, or
    //   (b) a Musketeer-namespaced execution log under `.musketeer/runs/<replayId>/`
    // For now, always write to the legacy location.
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
