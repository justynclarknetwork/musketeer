use crate::ui::RunSummary;

pub fn status_line(summary: &RunSummary) -> String {
    format!(
        "{} tasks {}/{} last update {}",
        summary.replay_id, summary.done, summary.total, summary.last
    )
}

pub fn ok_line(replay_id: &str) -> String {
    format!("check ready: {replay_id}")
}

pub fn check_failed_line(replay_id: &str) -> String {
    format!("check needs review: {replay_id}")
}
