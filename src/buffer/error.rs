use crate::file::BlockId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Pool exhausted: No buffers available")]
    PoolExhausted,
    #[error("Pin buffer for block {0} abort after too long waiting")]
    Abort(BlockId),
}
