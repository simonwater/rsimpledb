use std::{fmt, hash::Hash};

#[derive(Clone, Eq)]
pub struct BlockId {
    filename: String,
    blknum: i32,
}

impl BlockId {
    pub fn new(filename: String, blknum: i32) -> Self {
        BlockId { filename, blknum }
    }

    pub fn file_name(&self) -> &str {
        &self.filename
    }

    pub fn number(&self) -> i32 {
        self.blknum
    }
}

impl PartialEq for BlockId {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.blknum == other.blknum
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[file {}, block {}]", self.filename, self.blknum)
    }
}

impl fmt::Debug for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Hash for BlockId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("[file {}, block {}]", self.filename, self.blknum).hash(state);
    }
}
