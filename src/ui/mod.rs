pub mod interactive;
pub mod mode;
pub mod plain;
pub mod pretty;

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub replay_id: String,
    pub done: usize,
    pub total: usize,
    pub last: String,
}
