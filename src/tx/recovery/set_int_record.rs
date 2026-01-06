use crate::file::{BlockId, Page};
use crate::log::LogMgr;
use crate::tx::Transaction;
use crate::tx::recovery::{LogRecord, LogRecordType};
use std::fmt;

pub struct SetIntRecord {
    txnum: i32,
    offset: i32,
    val: i32,
    blk: BlockId,
}

impl SetIntRecord {
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
        let val = p.get_int(vpos);

        SetIntRecord {
            txnum,
            offset,
            val,
            blk,
        }
    }

    pub fn write_to_log(mut lm: LogMgr, txnum: i32, blk: &BlockId, offset: i32, val: i32) -> i32 {
        let tpos = 4;
        let fpos = tpos + 4;
        let bpos = fpos + Page::max_length(blk.file_name());
        let opos = bpos + 4;
        let vpos = opos + 4;
        let rec = vec![0u8; vpos + 4];
        let mut p = Page::from_bytes(rec);
        p.set_int(0, LogRecordType::SetInt as i32);
        p.set_int(tpos, txnum);
        p.set_string(fpos, blk.file_name());
        p.set_int(bpos, blk.number() as i32);
        p.set_int(opos, offset);
        p.set_int(vpos, val);
        lm.append(p.contents())
    }
}

impl LogRecord for SetIntRecord {
    fn op(&self) -> LogRecordType {
        LogRecordType::SetInt
    }

    fn tx_number(&self) -> i32 {
        self.txnum
    }

    fn undo(&self, tx: &mut Transaction) {
        tx.pin(&self.blk).unwrap();
        tx.set_int(&self.blk, self.offset as usize, self.val, false)
            .unwrap(); // don't log the undo!
        tx.unpin(&self.blk);
    }
}

impl fmt::Display for SetIntRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<SETINT {} {} {} {}>",
            self.txnum, self.blk, self.offset, self.val
        )
    }
}

impl fmt::Debug for SetIntRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
