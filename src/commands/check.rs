use std::env;

use anyhow::Context;

use crate::commands::util;
use crate::error::MusketeerError;
use crate::fs::{layout, read};
use crate::invariants::check::check_run;
use crate::model::run::Handoff;
use crate::model::verdict::Verdict;
use crate::musketeer_namespace;
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

    // Try new verdict location first, fall back to legacy handoff.yml
    let verdict_path = musketeer_namespace::verdict_path(&root, &replay_id);
    let rejected = if verdict_path.is_file() {
        let v: Verdict = read::read_yaml(&verdict_path)?;
        v.status == "reject"
    } else {
        // Legacy fallback: read from handoff.yml
        let handoff: Handoff = read::read_yaml(&layout::handoff_path(&root, &replay_id))?;
        handoff.verdict.as_deref() == Some("reject")
    };

    if rejected {
        return Err(MusketeerError::VerdictRejected("auditor rejected".to_string()).into());
    }

    if json_mode {
        output::emit_ok(json_mode, Some(&replay_id), serde_json::json!({}));
    } else {
        println!("check ok: {replay_id}");
    }
    Ok(())
}
