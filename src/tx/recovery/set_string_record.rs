use crate::file::{BlockId, Page};
use crate::log::LogMgr;
use crate::tx::Transaction;
use crate::tx::recovery::{LogRecord, LogRecordType};
use std::fmt;

pub struct SetStringRecord {
    txnum: i32,
    offset: i32,
    val: String,
    blk: BlockId,
}

impl SetStringRecord {
    pub fn from_page(p: &Page) -> Self {
        let tpos = 4; // Integer.BYTES
        let txnum = p.get_int(tpos);
        let fpos = tpos + 4;
        let filename = p.get_string(fpos);
        let bpos = fpos + Page::max_length(&filename);
        let blknum = p.get_int(bpos);
        let blk = BlockId::new(filename, blknum);
        let opos = bpos + 4;
        let offset = p.get_int(opos);
        let vpos = opos + 4;
        let val = p.get_string(vpos);

        SetStringRecord {
            txnum,
            offset,
            val,
            blk,
        }
    }

    pub fn write_to_log(mut lm: LogMgr, txnum: i32, blk: &BlockId, offset: i32, val: &str) -> i32 {
        let tpos = 4;
        let fpos = tpos + 4;
        let bpos = fpos + Page::max_length(blk.file_name());
        let opos = bpos + 4;
        let vpos = opos + 4;
        let val_len = Page::max_length(val);
        let rec = vec![0u8; vpos + val_len];
        let mut p = Page::from_bytes(rec);
        p.set_int(0, LogRecordType::SetString as i32);
        p.set_int(tpos, txnum);
        p.set_string(fpos, blk.file_name());
        p.set_int(bpos, blk.number() as i32);
        p.set_int(opos, offset);
        p.set_string(vpos, val);
        lm.append(p.contents())
    }
}

impl LogRecord for SetStringRecord {
    fn op(&self) -> LogRecordType {
        LogRecordType::SetString
    }

    fn tx_number(&self) -> i32 {
        self.txnum
    }

    fn undo(&self, tx: &mut Transaction) {
        let _ = tx.pin(&self.blk);
        let _ = tx.set_string(&self.blk, self.offset as usize, &self.val, false); // don't log the undo!
        tx.unpin(&self.blk);
    }
}

impl fmt::Display for SetStringRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<SETSTRING {} {} {} {}>",
            self.txnum, self.blk, self.offset, self.val
        )
    }
}

impl fmt::Debug for SetStringRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
