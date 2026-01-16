use crate::DbResult;
use crate::buffer::{Buffer, BufferMgr};
use crate::log::LogMgr;
use crate::tx::Transaction;
use crate::tx::recovery::{
    LogRecordType, checkpoint_record::CheckpointRecord, commit_record::CommitRecord,
    log_record::create_log_record, rollback_record::RollbackRecord, set_int_record::SetIntRecord,
    set_string_record::SetStringRecord, start_record::StartRecord,
};
use std::sync::{Arc, Mutex};

/// The recovery manager. Each transaction has its own recovery manager
pub struct RecoveryMgr {
    lm: LogMgr,
    bm: BufferMgr,
    txnum: i32,
}

impl RecoveryMgr {
    /// Create a recovery manager for the specified transaction
    pub fn new(txnum: i32, lm: LogMgr, bm: BufferMgr) -> DbResult<Self> {
        StartRecord::write_to_log(lm.clone(), txnum)?;
        Ok(RecoveryMgr { lm, bm, txnum })
    }

    /// Write a commit record to the log, and flushes it to disk
    pub fn commit(&mut self) -> DbResult<()> {
        self.bm.flush_all(self.txnum)?;
        let lsn = CommitRecord::write_to_log(self.lm.clone(), self.txnum)?;
        self.lm.flush(lsn)
    }

    /// Write a setint record to the log and return its lsn
    pub fn set_int(
        &mut self,
        buff: &Arc<Mutex<Buffer>>,
        offset: usize,
        _newval: i32,
    ) -> DbResult<i32> {
        let buffer = buff.lock().unwrap();
        let oldval = buffer.contents().get_int(offset);
        let blk = buffer.block().unwrap().clone();
        drop(buffer);
        SetIntRecord::write_to_log(self.lm.clone(), self.txnum, &blk, offset as i32, oldval)
    }

    /// Write a setstring record to the log and return its lsn
    pub fn set_string(
        &mut self,
        buff: &Arc<Mutex<Buffer>>,
        offset: usize,
        _newval: &str,
    ) -> DbResult<i32> {
        let buffer = buff.lock().unwrap();
        let oldval = buffer.contents().get_string(offset);
        let blk = buffer.block().unwrap().clone();
        drop(buffer);
        SetStringRecord::write_to_log(self.lm.clone(), self.txnum, &blk, offset as i32, &oldval)
    }
}

/// static methods for RecoveryMgr
impl RecoveryMgr {
    /// Write a rollback record to the log and flush it to disk
    pub fn rollback(mut lm: LogMgr, bm: BufferMgr, tx: &mut Transaction) -> DbResult<()> {
        Self::do_rollback(lm.clone(), tx)?;
        bm.flush_all(tx.txnum())?;
        let lsn = RollbackRecord::write_to_log(lm.clone(), tx.txnum())?;
        lm.flush(lsn)
    }

    /// Recover uncompleted transactions from the log
    pub fn recover(mut lm: LogMgr, bm: BufferMgr, tx: &mut Transaction) -> DbResult<()> {
        Self::do_recover(lm.clone(), tx)?;
        bm.flush_all(tx.txnum())?;
        let lsn = CheckpointRecord::write_to_log(lm.clone())?;
        lm.flush(lsn)
    }

    fn do_rollback(mut lm: LogMgr, tx: &mut Transaction) -> DbResult<()> {
        let mut iter = lm.iterator()?;

        while iter.has_next() {
            if let Some(result) = iter.next() {
                let rec = create_log_record(&result?);
                if rec.tx_number() == tx.txnum() {
                    if rec.op() == LogRecordType::Start {
                        return Ok(());
                    }
                    rec.undo(tx)?;
                }
            }
        }
        Ok(())
    }

    fn do_recover(mut lm: LogMgr, tx: &mut Transaction) -> DbResult<()> {
        use std::collections::HashSet;
        let mut finished_txs = HashSet::new();

        let mut iter = lm.iterator()?;

        while iter.has_next() {
            if let Some(result) = iter.next() {
                let rec = create_log_record(&result?);
                if rec.op() == LogRecordType::Checkpoint {
                    return Ok(());
                }
                if rec.op() == LogRecordType::Commit || rec.op() == LogRecordType::Rollback {
                    finished_txs.insert(rec.tx_number());
                } else if !finished_txs.contains(&rec.tx_number()) {
                    rec.undo(tx)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::DataBase;
    use crate::file::{BlockId, FileMgr, Page};
    use crate::util::TempFileGuard;
    use std::thread;

    #[test]
    fn recovery_mgr_test() {
        let db_dir = ".temp/recoverydb";
        let _guard = TempFileGuard::new(db_dir);
        let handle = thread::spawn(|| {
            let mut db: DataBase = DataBase::new(db_dir).unwrap();
            initialize(&mut db);
            // blk1回滚成功，blk2失败, 模拟系统崩溃
            modify(&mut db);
        });
        handle.join().unwrap();

        // 主线程模拟系统恢复
        let db: DataBase = DataBase::new(db_dir).unwrap();
        check_initial_values(&mut db.file_mgr());
    }

    fn initialize(db: &mut DataBase) {
        let mut fm = db.file_mgr();
        let blk1 = fm.append("testfile").unwrap();
        let blk2 = fm.append("testfile").unwrap();
        let mut tx1 = db.new_tx().unwrap();
        let mut tx2 = db.new_tx().unwrap();
        tx1.pin(&blk1).unwrap();
        tx2.pin(&blk2).unwrap();
        let mut pos = 0;
        for _ in 0..6 {
            tx1.set_int(&blk1, pos as usize, pos, true).unwrap();
            tx2.set_int(&blk2, pos as usize, pos, true).unwrap();
            pos = pos + 4; // Integer.BYTES
        }
        tx1.set_string(&blk1, 30, "abc", true).unwrap();
        tx2.set_string(&blk2, 30, "def", true).unwrap();
        tx1.commit().unwrap();
        tx2.commit().unwrap();

        // begin checking
        check_initial_values(&mut fm);
    }

    fn modify(db: &mut DataBase) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        let mut tx1 = db.new_tx().unwrap();
        let mut tx2 = db.new_tx().unwrap();
        tx1.pin(&blk1).unwrap();
        tx2.pin(&blk2).unwrap();
        let mut pos = 0;
        for _ in 0..6 {
            tx1.set_int(&blk1, pos as usize, pos * 1000, true).unwrap();
            tx2.set_int(&blk2, pos as usize, pos * 1000, true).unwrap();
            pos = pos + 4; // Integer.BYTES
        }
        tx1.set_string(&blk1, 30, "abc_modify", true).unwrap();
        tx2.set_string(&blk2, 30, "def_modify", true).unwrap();
        let bm = db.buffer_mgr();
        bm.flush_all(tx1.txnum()).unwrap();
        bm.flush_all(tx2.txnum()).unwrap();

        let mut fm = db.file_mgr();
        // begin checking
        check_modified_values(&mut fm);

        tx1.rollback().unwrap();
        check_partial_rollback_values(&mut fm);
    }

    fn read_pages(fm: &mut FileMgr, blk1: &BlockId, blk2: &BlockId) -> (Page, Page) {
        let mut p1 = Page::new(fm.block_size());
        let mut p2 = Page::new(fm.block_size());
        fm.read(blk1, &mut p1).unwrap();
        fm.read(blk2, &mut p2).unwrap();
        (p1, p2)
    }

    fn check_initial_values(fm: &mut FileMgr) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        let (p1, p2) = read_pages(fm, &blk1, &blk2);
        let mut pos = 0;
        for _ in 0..6 {
            assert_eq!(p1.get_int(pos), pos as i32);
            assert_eq!(p2.get_int(pos), pos as i32);
            pos = pos + 4; // Integer.BYTES
        }
        assert_eq!("abc", p1.get_string(30));
        assert_eq!("def", p2.get_string(30));
    }

    fn check_modified_values(fm: &mut FileMgr) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        let (p1, p2) = read_pages(fm, &blk1, &blk2);
        let mut pos = 0;
        for _ in 0..6 {
            assert_eq!(p1.get_int(pos), pos as i32 * 1000);
            assert_eq!(p2.get_int(pos), pos as i32 * 1000);
            pos = pos + 4; // Integer.BYTES
        }
        assert_eq!("abc_modify", p1.get_string(30));
        assert_eq!("def_modify", p2.get_string(30));
    }

    fn check_partial_rollback_values(fm: &mut FileMgr) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        let (p1, p2) = read_pages(fm, &blk1, &blk2);
        let mut pos: i32 = 0;
        for _ in 0..6 {
            assert_eq!(p1.get_int(pos as usize), pos);
            assert_eq!(p2.get_int(pos as usize), pos * 1000);
            pos = pos + 4; // Integer.BYTES
        }
        assert_eq!("abc", p1.get_string(30));
        assert_eq!("def_modify", p2.get_string(30));
    }
}
