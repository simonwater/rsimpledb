use crate::file::FileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Pool exhausted: No buffers available")]
    PoolExhausted,
    #[error("File storage error: {0}")]
    Storage(#[from] FileError),
}
