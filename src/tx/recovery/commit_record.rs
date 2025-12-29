use crate::file::Page;
use crate::log::LogMgr;
use crate::tx::Transaction;
use crate::tx::recovery::{LogRecord, LogRecordType};
use std::fmt;

pub struct CommitRecord {
    txnum: i32,
}

impl CommitRecord {
    pub fn from_page(p: &Page) -> Self {
        let tpos = 4; // Integer.BYTES
        CommitRecord {
            txnum: p.get_int(tpos),
        }
    }

    pub fn write_to_log(mut lm: LogMgr, txnum: i32) -> i32 {
        let rec = vec![0u8; 2 * 4]; // 2 * Integer.BYTES
        let mut p = Page::from_bytes(rec);
        p.set_int(0, LogRecordType::Commit as i32);
        p.set_int(4, txnum);
        lm.append(p.contents())
    }
}

impl LogRecord for CommitRecord {
    fn op(&self) -> LogRecordType {
        LogRecordType::Commit
    }

    fn tx_number(&self) -> i32 {
        self.txnum
    }

    fn undo(&self, _tx: &mut Transaction) {
        // Does nothing
    }
}

impl fmt::Display for CommitRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<COMMIT {}>", self.txnum)
    }
}

impl fmt::Debug for CommitRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
