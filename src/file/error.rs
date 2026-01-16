use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("{message}. IO error: {io_err}")]
    Io {
        #[source]
        io_err: io::Error,
        message: String,
    },
    #[error("File {path} already exists")]
    FileAlreadyExists { path: String },
    #[error("File {path} not found")]
    FileNotFound { path: String },
    #[error("Can not read file: {path}, block: {block_num}")]
    BlockReadFailed { path: String, block_num: i32 },
    #[error("Can not write file: {path}, block: {block_num}")]
    BlockWriteFailed { path: String, block_num: i32 },
}
