use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("SQL parse error at {line}:{col} line: {message}")]
    InvalidSyntax {
        message: String,
        line: usize,
        col: usize,
    },
    #[error("Unsupported keyword: {0}")]
    UnsupportedKeyword(String),
}
