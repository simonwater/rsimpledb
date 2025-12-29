use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct LockAbortException;

impl fmt::Display for LockAbortException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lock abort exception")
    }
}

impl Error for LockAbortException {}

