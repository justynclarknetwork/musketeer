use std::env;

use anyhow::Context;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::commands::util;
use crate::error::MusketeerError;
use crate::fs::write;
use crate::model::verdict::Verdict;
use crate::musketeer_namespace;
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

    // Write verdict to .musketeer/verdicts/<replayId>.verdict.yml
    musketeer_namespace::ensure_verdicts_dir(&root)?;

    let now = OffsetDateTime::now_utc();
    let timestamp = now.format(&Rfc3339).unwrap_or_else(|_| "unknown".to_string());

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
