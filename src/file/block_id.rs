pub struct BlockId {
    filename: String,
    blknum: u32,
}

impl BlockId {
    pub fn new(filename: String, blknum: u32) -> Self {
        BlockId { filename, blknum }
    }

    pub fn file_name(&self) -> &str {
        &self.filename
    }

    pub fn number(&self) -> u32 {
        self.blknum
    }
}
