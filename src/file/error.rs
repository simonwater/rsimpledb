use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("File {path} already exists")]
    FileAlreadyExists { path: String },
    #[error("Can not read file: {path}, block: {block_num}")]
    BlockReadFailed { path: String, block_num: i32 },
    #[error("Can not write file: {path}, block: {block_num}")]
    BlockWriteFailed { path: String, block_num: i32 },
}
