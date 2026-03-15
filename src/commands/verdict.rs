use std::env;

use anyhow::Context;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::MusketeerError;
use crate::fs::write;
use crate::model::verdict::Verdict;
use crate::musketeer_namespace;
use crate::output;
use crate::small_adapter;
use crate::workspace_mode::{self, WorkspaceContext};

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
    let ctx = workspace_mode::resolve(&root)?;
    let replay_id = workspace_mode::resolve_replay_id(&ctx, replay)?;

    // In SMALL-native mode, optionally read handoff for context (informational only).
    // Verdict is self-contained and does not depend on handoff content.
    if let WorkspaceContext::SmallNative { .. } = &ctx {
        if let Ok(handoff) = small_adapter::load_handoff(&root) {
            // Handoff note available for context: "{}"
            let _ = handoff.note;
        }
    }

    // Write verdict to .musketeer/verdicts/<replayId>.verdict.yml
    musketeer_namespace::ensure_verdicts_dir(&root)?;

    let now = OffsetDateTime::now_utc();
    let timestamp = now
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    let verdict = Verdict {
        kind: "musketeer_verdict".to_string(),
        musketeer_version: env!("CARGO_PKG_VERSION").to_string(),
        replay_id: replay_id.clone(),
        status: value.clone(),
        gate: role.clone(),
        reason,
        timestamp,
    };

    let verdict_path = musketeer_namespace::verdict_path(&root, &replay_id);
    write::write_yaml(&verdict_path, &verdict)?;

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
