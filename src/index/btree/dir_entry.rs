use crate::query::Constant;

/// A directory entry has two components: the number of the child block,
/// and the dataval of the first record in that block.
#[derive(Clone)]
pub struct DirEntry {
    dataval: Constant,
    blocknum: i32,
}

impl DirEntry {
    /// Creates a new entry for the specified dataval and block number.
    pub fn new(dataval: Constant, blocknum: i32) -> Self {
        DirEntry { dataval, blocknum }
    }

    /// Returns the dataval component of the entry
    pub fn data_val(&self) -> &Constant {
        &self.dataval
    }

    /// Returns the block number component of the entry
    pub fn block_number(&self) -> i32 {
        self.blocknum
    }
}
