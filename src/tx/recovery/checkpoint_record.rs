use crate::log::LogMgr;
use crate::tx::Transaction;
use crate::tx::recovery::{LogRecord, LogRecordType};

pub struct CheckpointRecord;

impl CheckpointRecord {
    pub fn new() -> Self {
        CheckpointRecord
    }

    pub fn write_to_log(mut lm: LogMgr) -> i32 {
        let rec = vec![0u8; 4]; // Integer.BYTES
        let mut p = crate::file::Page::from_bytes(rec);
        p.set_int(0, LogRecordType::Checkpoint as i32);
        lm.append(p.contents())
    }
}

impl LogRecord for CheckpointRecord {
    fn op(&self) -> LogRecordType {
        LogRecordType::Checkpoint
    }

    fn tx_number(&self) -> i32 {
        -1 // Checkpoint records don't have a transaction number
    }

    fn undo(&self, _tx: &mut Transaction) {
        // Does nothing
    }
}
