use crate::file::BlockId;
use crate::file::FileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Pool exhausted: No buffers available")]
    PoolExhausted,
    #[error("Pin buffer for block {0} abort after too long waiting")]
    Abort(BlockId),
    #[error("File storage error: {0}")]
    Storage(#[from] FileError),
}
