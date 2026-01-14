use crate::{buffer::BufferError, file::BlockId, log::LogError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TxError {
    #[error("Transaction deadlock: {0} ")]
    Deadlock(u32),
    #[error("Lock abort")]
    LockAbort,
    #[error("Transaction local buffer not found for the block: {0}")]
    LocalBufferNotFound(BlockId),
    #[error("Buffer manager error: {0}")]
    Buffer(#[from] BufferError),
    #[error("Log fail: {0}")]
    LogFailure(#[from] LogError),
}
