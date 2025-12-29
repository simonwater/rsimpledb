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
    pub fn new(txnum: i32, lm: LogMgr, bm: BufferMgr) -> Self {
        StartRecord::write_to_log(lm.clone(), txnum);

        RecoveryMgr { lm, bm, txnum }
    }

    /// Write a commit record to the log, and flushes it to disk
    pub fn commit(&mut self) {
        self.bm.flush_all(self.txnum);
        let lsn = CommitRecord::write_to_log(self.lm.clone(), self.txnum);
        self.lm.flush(lsn);
    }

    /// Write a setint record to the log and return its lsn
    pub fn set_int(&mut self, buff: &Arc<Mutex<Buffer>>, offset: usize, _newval: i32) -> i32 {
        let buffer = buff.lock().unwrap();
        let oldval = buffer.contents().get_int(offset);
        let blk = buffer.block().unwrap().clone();
        drop(buffer);
        SetIntRecord::write_to_log(self.lm.clone(), self.txnum, &blk, offset as i32, oldval)
    }

    /// Write a setstring record to the log and return its lsn
    pub fn set_string(&mut self, buff: &Arc<Mutex<Buffer>>, offset: usize, _newval: &str) -> i32 {
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
    pub fn rollback(mut lm: LogMgr, bm: BufferMgr, tx: &mut Transaction) {
        Self::do_rollback(lm.clone(), tx);
        bm.flush_all(tx.txnum());
        let lsn = RollbackRecord::write_to_log(lm.clone(), tx.txnum());
        lm.flush(lsn);
    }

    /// Recover uncompleted transactions from the log
    pub fn recover(mut lm: LogMgr, bm: BufferMgr, tx: &mut Transaction) {
        Self::do_recover(lm.clone(), tx);
        bm.flush_all(tx.txnum());
        let lsn = CheckpointRecord::write_to_log(lm.clone());
        lm.flush(lsn);
    }

    fn do_rollback(mut lm: LogMgr, tx: &mut Transaction) {
        let mut iter = lm.iterator();

        while iter.has_next() {
            if let Some(bytes) = iter.next() {
                let rec = create_log_record(&bytes);
                if rec.tx_number() == tx.txnum() {
                    if rec.op() == LogRecordType::Start {
                        return;
                    }
                    rec.undo(tx);
                }
            }
        }
    }

    fn do_recover(mut lm: LogMgr, tx: &mut Transaction) {
        use std::collections::HashSet;
        let mut finished_txs = HashSet::new();

        let mut iter = lm.iterator();

        while iter.has_next() {
            if let Some(bytes) = iter.next() {
                let rec = create_log_record(&bytes);
                if rec.op() == LogRecordType::Checkpoint {
                    return;
                }
                if rec.op() == LogRecordType::Commit || rec.op() == LogRecordType::Rollback {
                    finished_txs.insert(rec.tx_number());
                } else if !finished_txs.contains(&rec.tx_number()) {
                    rec.undo(tx);
                }
            }
        }
    }
}
