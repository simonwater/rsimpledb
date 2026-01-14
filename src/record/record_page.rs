use crate::DbResult;
use crate::file::BlockId;
use crate::record::Layout;
use crate::record::SqlTypes;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub const EMPTY: i32 = 0;
pub const USED: i32 = 1;

/// Store a record at a given location in a block
pub struct RecordPage {
    tx: Rc<RefCell<Transaction>>,
    blk: BlockId,
    layout: Arc<Layout>,
}

impl RecordPage {
    pub fn new(tx: Rc<RefCell<Transaction>>, blk: BlockId, layout: Arc<Layout>) -> Self {
        tx.borrow_mut().pin(&blk).expect("Failed to pin block");
        RecordPage { tx, blk, layout }
    }

    /// Return the integer value stored for the specified field of a specified slot
    pub fn get_int(&mut self, slot: i32, fldname: &str) -> DbResult<i32> {
        let fldpos = self.offset(slot) + self.layout.offset(fldname);
        self.tx.borrow_mut().get_int(&self.blk, fldpos as usize)
    }

    /// Return the string value stored for the specified field of the specified slot
    pub fn get_string(&mut self, slot: i32, fldname: &str) -> DbResult<String> {
        let fldpos = self.offset(slot) + self.layout.offset(fldname);
        self.tx.borrow_mut().get_string(&self.blk, fldpos as usize)
    }

    /// Store an integer at the specified field of the specified slot
    pub fn set_int(&mut self, slot: i32, fldname: &str, val: i32) -> DbResult<()> {
        let fldpos = self.offset(slot) + self.layout.offset(fldname);
        self.tx
            .borrow_mut()
            .set_int(&self.blk, fldpos as usize, val, true)
    }

    /// Store a string at the specified field of the specified slot
    pub fn set_string(&mut self, slot: i32, fldname: &str, val: &str) -> DbResult<()> {
        let fldpos = self.offset(slot) + self.layout.offset(fldname);
        self.tx
            .borrow_mut()
            .set_string(&self.blk, fldpos as usize, val, true)
    }

    pub fn delete(&mut self, slot: i32) -> DbResult<()> {
        self.set_flag(slot, EMPTY)
    }

    /// Use the layout to format a new block of records
    pub fn format(&mut self) -> DbResult<()> {
        let mut slot = 0;
        while self.is_valid_slot(slot) {
            self.tx
                .borrow_mut()
                .set_int(&self.blk, self.offset(slot) as usize, EMPTY, false)?;
            let sch = self.layout.schema();
            for fldname in sch.fields() {
                let fldpos = self.offset(slot) + self.layout.offset(fldname);
                if sch.ftype(fldname) == SqlTypes::INTEGER {
                    self.tx
                        .borrow_mut()
                        .set_int(&self.blk, fldpos as usize, 0, false)?;
                } else {
                    self.tx
                        .borrow_mut()
                        .set_string(&self.blk, fldpos as usize, "", false)?;
                }
            }
            slot += 1;
        }
        Ok(())
    }

    pub fn next_after(&mut self, slot: i32) -> DbResult<i32> {
        self.search_after(slot, USED)
    }

    pub fn insert_after(&mut self, slot: i32) -> DbResult<i32> {
        let newslot = self.search_after(slot, EMPTY)?;
        if newslot >= 0 {
            self.set_flag(newslot, USED)?;
        }
        Ok(newslot)
    }

    pub fn block(&self) -> &BlockId {
        &self.blk
    }

    fn set_flag(&mut self, slot: i32, flag: i32) -> DbResult<()> {
        self.tx
            .borrow_mut()
            .set_int(&self.blk, self.offset(slot) as usize, flag, true)
    }

    fn search_after(&mut self, slot: i32, flag: i32) -> DbResult<i32> {
        let mut current_slot = slot + 1;
        while self.is_valid_slot(current_slot) {
            let flag_val = {
                self.tx
                    .borrow_mut()
                    .get_int(&self.blk, self.offset(current_slot) as usize)?
            };
            if flag_val == flag {
                return Ok(current_slot);
            }
            current_slot += 1;
        }
        Ok(-1)
    }

    fn is_valid_slot(&self, slot: i32) -> bool {
        self.offset(slot + 1) <= self.tx.borrow_mut().block_size() as i32
    }

    fn offset(&self, slot: i32) -> i32 {
        slot * self.layout.slot_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataBase;
    use crate::record::{Layout, Schema};
    use crate::util::TempFileGuard;

    #[test]
    fn record_test() {
        let db_dir = ".temp/recorddb";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new(db_dir);
        let tx = Rc::new(RefCell::new(db.new_tx()));

        // 创建模式和布局
        let mut sch = Schema::new();
        sch.add_int_field("A");
        sch.add_string_field("B", 9);

        let layout = Layout::new(Arc::new(sch));
        assert_eq!(4, layout.offset("A"));
        assert_eq!(8, layout.offset("B"));
        assert_eq!(48, layout.slot_size());

        // 在文件末尾追加一个块并创建 RecordPage
        let blk = tx.borrow_mut().append("testfile").expect("append failed");
        let mut rp = RecordPage::new(Rc::clone(&tx), blk, Arc::new(layout));
        rp.format().expect("format failed");

        // 往slot中填充记录
        let mut slot = rp.insert_after(-1).expect("insert_after failed");
        let mut total = 0;
        let mut n = 1;
        while slot >= 0 {
            rp.set_int(slot, "A", n).expect("set_int failed");
            let s = format!("rec{}", n);
            rp.set_string(slot, "B", &s).expect("set_string failed");
            slot = rp.insert_after(slot).expect("insert_after failed");
            total = total + 1;
            n = n + 1;
        }
        assert_eq!(21, total);

        // 删除A为偶数的记录
        let mut count = 0i32;
        let mut slot = rp.next_after(-1).expect("next_after failed");
        while slot >= 0 {
            let a = rp.get_int(slot, "A").expect("get_int failed");
            if a % 2 == 0 {
                count += 1;
                rp.delete(slot).expect("delete failed");
            }
            slot = rp.next_after(slot).expect("next_after failed");
        }
        assert_eq!(10, count);

        let mut slot = rp.next_after(-1).expect("next_after failed");
        let mut n = 1;
        while slot >= 0 {
            let a = rp.get_int(slot, "A").expect("get_int failed");
            let b = rp.get_string(slot, "B").expect("get_string failed");
            assert_eq!(n, a);
            assert_eq!(format!("rec{}", n), b);
            slot = rp.next_after(slot).expect("next_after failed");
            n = n + 2;
        }

        // 显式 drop RecordPage（会在 Drop 中 unpin）并提交事务
        drop(rp);
        tx.borrow_mut().commit();
    }
}
