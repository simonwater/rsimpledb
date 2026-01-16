use crate::file::BlockId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TxError {
    #[error("Transaction deadlock: {0} ")]
    Deadlock(i32),
    #[error("Transaction {0} acquire {1} lock abort after too long waiting")]
    LockAbort(i32, &'static str),
    #[error("Transaction {0} local buffer not found for the block: {1}")]
    LocalBufferNotFound(i32, BlockId),
}
