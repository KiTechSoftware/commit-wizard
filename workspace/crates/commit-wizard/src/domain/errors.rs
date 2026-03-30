use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("summary cannot be empty")]
    EmptySummary,
    #[error("summary too long (max {0} chars)")]
    SummaryTooLong(usize),
}
