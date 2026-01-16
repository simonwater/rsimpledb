use std::fmt;

use crate::DbResult;
use crate::file::Page;
use crate::tx::Transaction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRecordType {
    Checkpoint = 0,
    Start = 1,
    Commit = 2,
    Rollback = 3,
    SetInt = 4,
    SetString = 5,
}

pub trait LogRecord: fmt::Display + fmt::Debug {
    fn op(&self) -> LogRecordType;
    fn tx_number(&self) -> i32;
    fn undo(&self, _tx: &mut Transaction) -> DbResult<()> {
        Ok(())
    }
}

pub fn create_log_record(bytes: &[u8]) -> Box<dyn LogRecord> {
    let p = Page::from_bytes(bytes.to_vec());
    let op_type = p.get_int(0);

    match op_type {
        0 => Box::new(crate::tx::recovery::checkpoint_record::CheckpointRecord::new()),
        1 => Box::new(crate::tx::recovery::start_record::StartRecord::from_page(
            &p,
        )),
        2 => Box::new(crate::tx::recovery::commit_record::CommitRecord::from_page(
            &p,
        )),
        3 => Box::new(crate::tx::recovery::rollback_record::RollbackRecord::from_page(&p)),
        4 => Box::new(crate::tx::recovery::set_int_record::SetIntRecord::from_page(&p)),
        5 => Box::new(crate::tx::recovery::set_string_record::SetStringRecord::from_page(&p)),
        _ => panic!("Unknown log record type: {}", op_type),
    }
}
