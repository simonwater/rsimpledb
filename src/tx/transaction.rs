use crate::buffer::BufferMgr;
use crate::file::{BlockId, FileMgr};
use crate::log::LogMgr;
use crate::tx::buffer_list::BufferList;
use crate::tx::concurrency::{ConcurrencyMgr, LockAbortException, LockTable};
use crate::tx::recovery::RecoveryMgr;
use std::sync::atomic::{AtomicI32, Ordering};

static NEXT_TX_NUM: AtomicI32 = AtomicI32::new(0);
const END_OF_FILE: i32 = -1;

/// Provide transaction management for clients
pub struct Transaction {
    recovery_mgr: RecoveryMgr,
    concur_mgr: ConcurrencyMgr,
    lm: LogMgr,
    bm: BufferMgr,
    fm: FileMgr,
    txnum: i32,
    mybuffers: BufferList,
}

impl Transaction {
    /// Create a new transaction and its associated recovery and concurrency managers
    pub fn new(fm: FileMgr, lm: LogMgr, bm: BufferMgr, lt: LockTable) -> Self {
        let txnum = Self::next_tx_number();
        let recovery_mgr = RecoveryMgr::new(txnum, lm.clone(), bm.clone());

        Self {
            recovery_mgr,
            concur_mgr: ConcurrencyMgr::new(lt),
            bm: bm.clone(),
            lm,
            fm,
            txnum,
            mybuffers: BufferList::new(bm.clone()),
        }
    }

    /// Commit the current transaction
    pub fn commit(&mut self) {
        self.recovery_mgr.commit();
        //println!("transaction {} committed", self.txnum);
        self.concur_mgr.release();
        self.mybuffers.unpin_all();
    }

    /// Rollback the current transaction
    pub fn rollback(&mut self) {
        RecoveryMgr::rollback(self.lm.clone(), self.bm.clone(), self);
        //println!("transaction {} rolled back", self.txnum);
        self.concur_mgr.release();
        self.mybuffers.unpin_all();
    }

    /// Recover uncompleted transactions
    pub fn recover(&mut self) {
        self.bm.flush_all(self.txnum);
        RecoveryMgr::recover(self.lm.clone(), self.bm.clone(), self);
    }

    /// Pin the specified block
    pub fn pin(&mut self, blk: &BlockId) -> Result<(), crate::buffer::BufferAbortException> {
        self.mybuffers.pin(blk)
    }

    /// Unpin the specified block
    pub fn unpin(&mut self, blk: &BlockId) {
        self.mybuffers.unpin(blk);
    }

    /// Return the integer value stored at the specified offset
    pub fn get_int(&mut self, blk: &BlockId, offset: usize) -> Result<i32, LockAbortException> {
        self.concur_mgr.s_lock(blk)?;
        let buff = self
            .mybuffers
            .get_buffer(blk)
            .ok_or_else(|| LockAbortException)?;
        let buffer = buff.lock().unwrap();
        Ok(buffer.contents().get_int(offset))
    }

    /// Return the string value stored at the specified offset
    pub fn get_string(
        &mut self,
        blk: &BlockId,
        offset: usize,
    ) -> Result<String, LockAbortException> {
        self.concur_mgr.s_lock(blk)?;
        let buff = self
            .mybuffers
            .get_buffer(blk)
            .ok_or_else(|| LockAbortException)?;
        let buffer = buff.lock().unwrap();
        Ok(buffer.contents().get_string(offset))
    }

    /// Store an integer at the specified offset
    pub fn set_int(
        &mut self,
        blk: &BlockId,
        offset: usize,
        val: i32,
        ok_to_log: bool,
    ) -> Result<(), LockAbortException> {
        self.concur_mgr.x_lock(blk)?;
        let buff = self
            .mybuffers
            .get_buffer(blk)
            .ok_or_else(|| LockAbortException)?;
        let lsn = if ok_to_log {
            self.recovery_mgr.set_int(&buff, offset, val)
        } else {
            -1
        };
        let mut buffer = buff.lock().unwrap();
        buffer.contents_mut().set_int(offset, val);
        buffer.set_modified(self.txnum, lsn);
        Ok(())
    }

    /// Store a string at the specified offset
    pub fn set_string(
        &mut self,
        blk: &BlockId,
        offset: usize,
        val: &str,
        ok_to_log: bool,
    ) -> Result<(), LockAbortException> {
        self.concur_mgr.x_lock(blk)?;
        let buff = self
            .mybuffers
            .get_buffer(blk)
            .ok_or_else(|| LockAbortException)?;
        let lsn = if ok_to_log {
            self.recovery_mgr.set_string(&buff, offset, val)
        } else {
            -1
        };
        let mut buffer = buff.lock().unwrap();
        buffer.contents_mut().set_string(offset, val);
        buffer.set_modified(self.txnum, lsn);
        Ok(())
    }

    /// Return the number of blocks in the specified file
    pub fn size(&mut self, filename: &str) -> Result<usize, LockAbortException> {
        let dummyblk = BlockId::new(filename.to_string(), END_OF_FILE);
        self.concur_mgr.s_lock(&dummyblk)?;
        Ok(self.fm.length(filename))
    }

    /// Append a new block to the end of the specified file
    pub fn append(&mut self, filename: &str) -> Result<BlockId, LockAbortException> {
        let dummyblk = BlockId::new(filename.to_string(), END_OF_FILE);
        self.concur_mgr.x_lock(&dummyblk)?;
        Ok(self.fm.append(filename))
    }

    pub fn block_size(&self) -> usize {
        self.fm.block_size()
    }

    pub fn available_buffs(&self) -> i32 {
        self.bm.available() as i32
    }

    pub fn txnum(&self) -> i32 {
        self.txnum
    }

    fn next_tx_number() -> i32 {
        NEXT_TX_NUM.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
pub mod tests {

    use crate::DataBase;
    use crate::file::BlockId;
    use crate::util::TempFileGuard;

    #[test]
    fn tx_simple_test() {
        let db_dir = ".temp/txdb1";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new(db_dir);
        let mut fm = db.file_mgr();
        let blk = fm.append("testfile");
        tx_test(db, blk);
    }

    fn tx_test(db: DataBase, blk: BlockId) {
        let mut tx1 = db.new_tx();
        tx1.pin(&blk).unwrap();
        tx1.set_int(&blk, 0, 123, true).unwrap();
        tx1.set_string(&blk, 10, "hello", true).unwrap();
        assert_eq!(123, tx1.get_int(&blk, 0).unwrap());
        assert_eq!("hello".to_string(), tx1.get_string(&blk, 10).unwrap());
        tx1.commit();

        let mut tx2 = db.new_tx();
        tx2.pin(&blk).unwrap();
        let ival = tx2.get_int(&blk, 0).unwrap();
        let sval = tx2.get_string(&blk, 10).unwrap();
        assert_eq!(123, ival);
        assert_eq!("hello".to_string(), sval);
        tx2.set_int(&blk, 0, 456, true).unwrap();
        tx2.set_string(&blk, 10, "world", true).unwrap();
        assert_eq!(456, tx2.get_int(&blk, 0).unwrap());
        assert_eq!("world".to_string(), tx2.get_string(&blk, 10).unwrap());
        tx2.commit();

        let mut tx3 = db.new_tx();
        tx3.pin(&blk).unwrap();
        assert_eq!(456, tx3.get_int(&blk, 0).unwrap());
        assert_eq!("world".to_string(), tx3.get_string(&blk, 10).unwrap());
        tx3.set_int(&blk, 0, 999, true).unwrap();
        assert_eq!(999, tx3.get_int(&blk, 0).unwrap());
        tx3.rollback();

        let mut tx4 = db.new_tx();
        tx4.pin(&blk).unwrap();
        assert_eq!(456, tx4.get_int(&blk, 0).unwrap());
        assert_eq!("world".to_string(), tx4.get_string(&blk, 10).unwrap());
        tx4.commit();
    }
}
