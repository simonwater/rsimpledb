use crate::file::FileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("Invalid LSN {0}")]
    InvalidLSN(u64),
    #[error("Log is corrupted")]
    LogCorrupted,
    #[error("Log storage access failed: {0}")]
    Storage(#[from] FileError),
}
