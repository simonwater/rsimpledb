use std::fmt;

pub struct BlockId {
    filename: String,
    blknum: usize,
}

impl BlockId {
    pub fn new(filename: String, blknum: usize) -> Self {
        BlockId { filename, blknum }
    }

    pub fn file_name(&self) -> &str {
        &self.filename
    }

    pub fn number(&self) -> usize {
        self.blknum
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
