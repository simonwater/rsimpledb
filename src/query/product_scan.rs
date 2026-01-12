use crate::query::{Constant, Scan};

/// The scan class corresponding to the product relational algebra operator
pub struct ProductScan {
    s1: Box<dyn Scan>,
    s2: Box<dyn Scan>,
}

impl ProductScan {
    /// Create a product scan having the two underlying scans
    pub fn new(s1: Box<dyn Scan>, s2: Box<dyn Scan>) -> Self {
        let mut ps = ProductScan { s1, s2 };
        ps.before_first();
        ps
    }
}

impl Scan for ProductScan {
    fn before_first(&mut self) {
        self.s1.before_first();
        self.s1.next();
        self.s2.before_first();
    }

    fn next(&mut self) -> bool {
        if self.s2.next() {
            true
        } else {
            self.s2.before_first();
            self.s2.next() && self.s1.next()
        }
    }

    fn get_int(&mut self, fldname: &str) -> i32 {
        if self.s1.has_field(fldname) {
            self.s1.get_int(fldname)
        } else {
            self.s2.get_int(fldname)
        }
    }

    fn get_string(&mut self, fldname: &str) -> String {
        if self.s1.has_field(fldname) {
            self.s1.get_string(fldname)
        } else {
            self.s2.get_string(fldname)
        }
    }

    fn get_val(&mut self, fldname: &str) -> Constant {
        if self.s1.has_field(fldname) {
            self.s1.get_val(fldname)
        } else {
            self.s2.get_val(fldname)
        }
    }

    fn has_field(&self, fldname: &str) -> bool {
        self.s1.has_field(fldname) || self.s2.has_field(fldname)
    }

    fn close(&mut self) {
        self.s1.close();
        self.s2.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataBase;
    use crate::query::UpdateScan;
    use crate::record::{Layout, Schema, TableScan, layout};
    use crate::util::TempFileGuard;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn product_test() {
        let db_dir = ".temp/productscan_db";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new(db_dir);
        let mut sch1 = Schema::new();
        sch1.add_int_field("A");
        sch1.add_string_field("B", 9);
        let layout1 = Rc::new(Layout::new(Rc::new(sch1)));
        let mut sch2 = Schema::new();
        sch2.add_int_field("C");
        sch2.add_string_field("D", 9);
        let layout2 = Rc::new(Layout::new(Rc::new(sch2)));

        let tx1 = Rc::new(RefCell::new(db.new_tx()));
        let mut ts1 = TableScan::new(Rc::clone(&tx1), "T1", Rc::clone(&layout1));
        for i in 1..=10 {
            ts1.insert();
            ts1.set_int("A", i);
            ts1.set_string("B", &format!("t1_rec{}", i));
        }
        ts1.close();

        let mut ts2 = TableScan::new(Rc::clone(&tx1), "T2", Rc::clone(&layout2));
        for i in 1..=5 {
            ts2.insert();
            ts2.set_int("C", i * 2);
            ts2.set_string("D", &format!("t2_rec{}", i * 2));
        }
        ts2.close();
        tx1.borrow_mut().commit();

        let tx2 = Rc::new(RefCell::new(db.new_tx()));
        let ts1 = TableScan::new(Rc::clone(&tx2), "T1", Rc::clone(&layout1));
        let ts2 = TableScan::new(Rc::clone(&tx2), "T2", Rc::clone(&layout2));
        let mut ps = ProductScan::new(Box::new(ts1), Box::new(ts2));
        let mut i = 0;
        while ps.next() {
            let a = ps.get_int("A");
            let b = ps.get_string("B");
            let c = ps.get_int("C");
            let d = ps.get_string("D");
            assert_eq!(a, i / 5 + 1);
            assert_eq!(b, format!("t1_rec{}", i / 5 + 1));
            assert_eq!(c, (i % 5 + 1) * 2);
            assert_eq!(d, format!("t2_rec{}", (i % 5 + 1) * 2));
            i += 1;
        }
        ps.close();
        tx2.borrow_mut().commit();
    }
}
