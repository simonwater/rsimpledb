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

    pub fn update_dir_page(
        &mut self,
        right_mask: i32,
        mask_len: i32,
        new_bucket_blknum: i32,
    ) -> DbResult<()> {
        let full_mask = (1 << mask_len) - 1;
        let blocksize = self.tx.borrow().block_size() as i32;
        let num_buckets = blocksize / 4;
        for bucket in 0..num_buckets {
            if (bucket & full_mask) == right_mask {
                self.set_bucket_blknum(bucket, new_bucket_blknum)?;
            }
        }
        Ok(())
    }
}
