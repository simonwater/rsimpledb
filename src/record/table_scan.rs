use crate::file::BlockId;
use crate::query::{Constant, Scan, UpdateScan};
use crate::record::SqlTypes;
use crate::record::{Layout, RID, RecordPage};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct TableScan {
    tx: Rc<RefCell<Transaction>>,
    layout: Arc<Layout>,
    rp: Option<RecordPage>,
    filename: String,
    currentslot: i32,
}

impl TableScan {
    pub fn new(tx: Rc<RefCell<Transaction>>, tblname: &str, layout: Arc<Layout>) -> Self {
        let filename = format!("{}.tbl", tblname);
        let size = tx.borrow_mut().size(&filename).unwrap();
        let mut scan = TableScan {
            tx,
            layout,
            rp: None,
            filename,
            currentslot: -1,
        };
        if size == 0 {
            scan.move_to_new_block();
        } else {
            scan.move_to_block(0);
        }
        scan
    }

    fn move_to_block(&mut self, blknum: i32) {
        self.close();
        let blk = BlockId::new(self.filename.to_string(), blknum);
        self.rp = Some(RecordPage::new(
            Rc::clone(&self.tx),
            blk,
            Arc::clone(&self.layout),
        ));
    }

    fn move_to_new_block(&mut self) {
        self.close();
        let blk = self.tx.borrow_mut().append(&self.filename).unwrap();
        let mut rp = RecordPage::new(Rc::clone(&self.tx), blk, Arc::clone(&self.layout));
        rp.format().unwrap();
        self.rp = Some(rp)
    }

    fn at_last_block(&mut self) -> bool {
        if let Some(ref rp) = self.rp {
            let size = self.tx.borrow_mut().size(&self.filename).unwrap() as i32;
            rp.block().number() == size - 1
        } else {
            false
        }
    }
}

impl Scan for TableScan {
    fn before_first(&mut self) {
        self.move_to_block(0);
        self.currentslot = -1;
    }

    fn next(&mut self) -> bool {
        if self.rp.is_none() {
            return false;
        }
        self.currentslot = self
            .rp
            .as_mut()
            .unwrap()
            .next_after(self.currentslot)
            .unwrap();

        while self.currentslot < 0 {
            if self.at_last_block() {
                return false;
            }
            let next_blk = self.rp.as_mut().unwrap().block().number() + 1;
            self.move_to_block(next_blk);
            if let Some(ref mut new_rp) = self.rp {
                self.currentslot = new_rp.next_after(-1).unwrap();
            }
        }
        self.currentslot >= 0
    }

    fn get_int(&mut self, fldname: &str) -> i32 {
        if let Some(ref mut rp) = self.rp {
            rp.get_int(self.currentslot, fldname).unwrap()
        } else {
            0
        }
    }

    fn get_string(&mut self, fldname: &str) -> String {
        if let Some(ref mut rp) = self.rp {
            rp.get_string(self.currentslot, fldname).unwrap()
        } else {
            String::new()
        }
    }

    fn get_val(&mut self, fldname: &str) -> Constant {
        if self.layout.schema().ftype(fldname) == SqlTypes::INTEGER {
            Constant::from_int(self.get_int(fldname))
        } else {
            Constant::from_string(self.get_string(fldname))
        }
    }

    fn has_field(&self, fldname: &str) -> bool {
        self.layout.schema().has_field(fldname)
    }

    fn close(&mut self) {
        self.rp = None;
    }

    fn as_update_scan(&mut self) -> Option<&mut dyn UpdateScan> {
        Some(self as &mut dyn UpdateScan)
    }
}

impl UpdateScan for TableScan {
    fn set_int(&mut self, fldname: &str, val: i32) {
        if let Some(ref mut rp) = self.rp {
            rp.set_int(self.currentslot, fldname, val).unwrap();
        }
    }

    fn set_string(&mut self, fldname: &str, val: &str) {
        if let Some(ref mut rp) = self.rp {
            rp.set_string(self.currentslot, fldname, val).unwrap();
        }
    }

    fn set_val(&mut self, fldname: &str, val: &Constant) {
        if self.layout.schema().ftype(fldname) == SqlTypes::INTEGER {
            if let Some(i) = val.as_int() {
                self.set_int(fldname, i);
            }
        } else {
            if let Some(s) = val.as_string() {
                self.set_string(fldname, s);
            }
        }
    }

    fn insert(&mut self) {
        if self.rp.is_none() {
            return;
        }
        self.currentslot = self
            .rp
            .as_mut()
            .unwrap()
            .insert_after(self.currentslot)
            .unwrap();
        while self.currentslot < 0 {
            if self.at_last_block() {
                self.move_to_new_block();
            } else {
                let next_blk = self.rp.as_mut().unwrap().block().number() + 1;
                self.move_to_block(next_blk);
            }
            if let Some(ref mut new_rp) = self.rp {
                self.currentslot = new_rp.insert_after(-1).unwrap();
            }
        }
    }

    fn delete(&mut self) {
        if let Some(ref mut rp) = self.rp {
            rp.delete(self.currentslot).unwrap();
        }
    }

    fn get_rid(&mut self) -> RID {
        if let Some(ref rp) = self.rp {
            RID::new(rp.block().number(), self.currentslot)
        } else {
            RID::new(-1, -1)
        }
    }

    fn move_to_rid(&mut self, rid: &RID) {
        let blk = BlockId::new(self.filename.clone(), rid.block_number());
        self.rp = Some(RecordPage::new(
            Rc::clone(&self.tx),
            blk,
            Arc::clone(&self.layout),
        ));
        self.currentslot = rid.slot();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataBase;
    use crate::record::Schema;
    use crate::util::TempFileGuard;

    #[test]
    fn table_scan_test() {
        let db_dir = ".temp/tablescan_db";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new(db_dir);
        let tx = Rc::new(RefCell::new(db.new_tx()));
        let mut sch = Schema::new();
        sch.add_int_field("A");
        sch.add_string_field("B", 9);
        let layout = Layout::new(sch);

        // 新增记录
        let mut ts = TableScan::new(Rc::clone(&tx), "T", Arc::new(layout));
        for i in 1..=100 {
            ts.insert();
            ts.set_int("A", i);
            ts.set_string("B", &format!("rec{}", i));
        }

        // 删除A为偶数的记录
        let mut count = 0i32;
        ts.before_first();
        while ts.next() {
            let a = ts.get_int("A");
            if a % 2 == 0 {
                count += 1;
                ts.delete();
            }
        }
        assert_eq!(50, count);

        // 剩余记录数
        ts.before_first();
        let mut n = 1;
        while ts.next() {
            let a = ts.get_int("A");
            let b = ts.get_string("B");
            assert_eq!(n, a);
            assert_eq!(format!("rec{}", n), b);
            n += 2;
        }
        //ts.close();
        tx.borrow_mut().commit();
    }
}
