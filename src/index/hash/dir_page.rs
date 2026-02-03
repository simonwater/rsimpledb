use crate::DbResult;
use crate::file::BlockId;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

pub struct DirPage {
    tx: Rc<RefCell<Transaction>>,
    blk: BlockId, // DirPage[bucket] -> block number
}
impl DirPage {
    pub fn new(tx: Rc<RefCell<Transaction>>, blk: BlockId) -> DbResult<Self> {
        tx.borrow_mut().pin(&blk)?;
        Ok(DirPage { tx, blk })
    }

    pub fn get_bucket_blknum(&mut self, bucket: i32) -> DbResult<i32> {
        self.tx.borrow_mut().get_int(&self.blk, 4 * bucket as usize)
    }

    pub fn set_bucket_blknum(&mut self, bucket: i32, blknum: i32) -> DbResult<()> {
        self.tx
            .borrow_mut()
            .set_int(&self.blk, 4 * bucket as usize, blknum, true)
    }
}
