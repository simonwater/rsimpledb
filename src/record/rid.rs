/// A record identifier, consisting of the block number
/// in which the record resides and the record's slot number
#[derive(Eq)]
pub struct RID {
    blknum: i32,
    slot: i32,
}

impl RID {
    pub fn new(blknum: i32, slot: i32) -> Self {
        RID { blknum, slot }
    }

    pub fn block_number(&self) -> i32 {
        self.blknum
    }

    pub fn slot(&self) -> i32 {
        self.slot
    }

    pub fn block_id(&self, filename: &str) -> crate::file::BlockId {
        crate::file::BlockId::new(filename.to_string(), self.blknum)
    }
}

impl PartialEq for RID {
    fn eq(&self, other: &Self) -> bool {
        self.blknum == other.blknum && self.slot == other.slot
    }
}
