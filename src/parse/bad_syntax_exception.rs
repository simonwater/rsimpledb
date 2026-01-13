use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct BadSyntaxException;

impl fmt::Display for BadSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad syntax exception")
    }
}

impl Error for BadSyntaxException {}

impl std::default::Default for BadSyntaxException {
    fn default() -> Self {
        BadSyntaxException
    }
}

