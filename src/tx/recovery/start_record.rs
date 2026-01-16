use crate::DbResult;
use crate::file::Page;
use crate::log::LogMgr;
use crate::tx::recovery::{LogRecord, LogRecordType};
use std::fmt;

pub struct StartRecord {
    txnum: i32,
}

impl StartRecord {
    pub fn from_page(p: &Page) -> Self {
        let tpos = 4; // Integer.BYTES
        StartRecord {
            txnum: p.get_int(tpos),
        }
    }

    pub fn write_to_log(mut lm: LogMgr, txnum: i32) -> DbResult<i32> {
        let rec = vec![0u8; 2 * 4]; // 2 * Integer.BYTES
        let mut p = Page::from_bytes(rec);
        p.set_int(0, LogRecordType::Start as i32);
        p.set_int(4, txnum);
        lm.append(p.contents())
    }
}

impl LogRecord for StartRecord {
    fn op(&self) -> LogRecordType {
        LogRecordType::Start
    }

    fn tx_number(&self) -> i32 {
        self.txnum
    }
}

impl fmt::Display for StartRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<START {}>", self.txnum)
    }
}

impl fmt::Debug for StartRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
