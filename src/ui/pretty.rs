use comfy_table::{Cell, ContentArrangement, Table};
use owo_colors::OwoColorize;

use crate::ui::RunSummary;

pub fn status_table(summaries: &[RunSummary]) -> String {
    let mut table = Table::new();
    table
        .set_header(vec!["replay_id", "done", "total", "last update"])
        .set_content_arrangement(ContentArrangement::Dynamic);
    for summary in summaries {
        table.add_row(vec![
            Cell::new(&summary.replay_id),
            Cell::new(summary.done),
            Cell::new(summary.total),
            Cell::new(&summary.last),
        ]);
    }
    table.to_string()
}

pub fn ok_marker() -> String {
    "READY".green().to_string()
}

pub fn fail_marker() -> String {
    "NEEDS REVIEW".red().bold().to_string()
}

pub fn ok_line(replay_id: &str) -> String {
    format!("{} check ready: {}", ok_marker(), replay_id)
}

pub fn fail_line(replay_id: &str) -> String {
    format!("{} check needs review: {}", fail_marker(), replay_id)
}
