//! `musketeer migrate` command handler.

use std::env;

use anyhow::Context;

use crate::error::MusketeerError;
use crate::migration::{self, MigrationState};
use crate::output;

pub fn run(json_mode: bool, dry_run: bool, force: bool) -> anyhow::Result<()> {
    let root = env::current_dir().context("failed to resolve current dir")?;

    let state = migration::detect_migration_state(&root);

    match state {
        MigrationState::SmallNative => {
            if json_mode {
                output::emit_ok(
                    json_mode,
                    None,
                    serde_json::json!({"migration": "already_small_native"}),
                );
            } else {
                println!("Workspace is already SMALL-native. Nothing to migrate.");
            }
            return Ok(());
        }
        MigrationState::Empty => {
            return Err(MusketeerError::WorkspaceMissing(
                "no workspace detected; nothing to migrate".to_string(),
            )
            .into());
        }
        MigrationState::Mixed => {
            if !force {
                return Err(MusketeerError::InvalidInput(
                    "mixed workspace detected (both .small/ and legacy artifacts). Use --force to migrate anyway.".to_string(),
                )
                .into());
            }
            eprintln!("[warn] Mixed workspace detected. Proceeding with --force.");
        }
        MigrationState::Legacy => {
            // Normal case, proceed
        }
    }

    let plan = migration::plan_migration(&root)?;

    if dry_run {
        if json_mode {
            let plan_json = serde_json::to_value(&plan).context("failed to serialize plan")?;
            output::emit_ok(
                json_mode,
                Some(&plan.replay_id),
                serde_json::json!({"migration": "dry_run", "plan": plan_json}),
            );
        } else {
            println!("=== Migration Dry Run ===");
            println!("Active replay ID: {}", plan.replay_id);
            println!("Total runs found: {}", plan.all_run_ids.len());
            println!("Artifacts to convert:");
            for artifact in &plan.artifacts_found {
                println!("  {} (run: {})", artifact.path, artifact.run_id);
            }
            println!("Archive directory: .musketeer/legacy/{}/", plan.archive_timestamp);
            if plan.all_run_ids.len() > 1 {
                let others: Vec<&str> = plan
                    .all_run_ids
                    .iter()
                    .filter(|id| *id != &plan.replay_id)
                    .map(|s| s.as_str())
                    .collect();
                println!(
                    "Other runs (archive only): {}",
                    others.join(", ")
                );
            }
            println!("\nNo files were created (dry run).");
        }
        return Ok(());
    }

    let report = migration::execute_migration(&root, &plan)?;

    // Verify
    let post_state = migration::detect_migration_state(&root);
    if post_state != MigrationState::SmallNative {
        eprintln!(
            "[warn] Post-migration verification: workspace is {:?}, expected SmallNative",
            post_state
        );
    }

    if json_mode {
        let report_json =
            serde_json::to_value(&report).context("failed to serialize report")?;
        output::emit_ok(
            json_mode,
            Some(&report.replay_id),
            serde_json::json!({"migration": "completed", "report": report_json}),
        );
    } else {
        println!("=== Migration Complete ===");
        println!("Replay ID: {}", report.replay_id);
        println!("Files converted: {}", report.files_converted.len());
        println!("Files archived: {}", report.files_archived.len());
        if !report.fields_ambiguous.is_empty() {
            println!("Ambiguous fields:");
            for f in &report.fields_ambiguous {
                println!(
                    "  {}.{}: {} ({})",
                    f.source, f.field, f.reason, f.action
                );
            }
        }
        if !report.warnings.is_empty() {
            println!("Warnings:");
            for w in &report.warnings {
                println!("  {}", w);
            }
        }
        println!("Report: .musketeer/migration-report.yml");
        if post_state == MigrationState::SmallNative {
            println!("Verification: workspace is now SMALL-native.");
        }
    }

    Ok(())
}
