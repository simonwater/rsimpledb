use crate::buffer::BufferError;
use crate::metadata::MetadataError;
use crate::parse::ParseError;
use crate::plan::PlanError;
use crate::tx::TxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Buffer manager error: {0}")]
    Buffer(#[from] BufferError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TxError),

    #[error("Meta data manager error: {0}")]
    Metadata(#[from] MetadataError),

    #[error("Parser error: {0}")]
    Parser(#[from] ParseError),

    #[error("Planner error: {0}")]
    Planner(#[from] PlanError),

    // 用于快速抛出的通用错误
    #[error("System internal error: {0}")]
    Internal(String),
}

pub type DbResult<T> = std::result::Result<T, DbError>;
