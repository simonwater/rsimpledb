use crate::DataBase;
use crate::query::{
    Constant, Expression, Predicate, ProductScan, ProjectScan, Scan, SelectScan, Term, UpdateScan,
};
use crate::record::{Layout, Schema, TableScan};
use crate::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[test]
fn scan_test1() {
    let db_dir = ".temp/scantest1";
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new(db_dir).unwrap();
    let mut sch = Schema::new();
    sch.add_int_field("A");
    sch.add_string_field("B", 9);
    let layout = Arc::new(Layout::new(Arc::new(sch)));
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));

    let mut s1 = TableScan::new(Rc::clone(&tx), "T", Arc::clone(&layout)).unwrap();
    for i in 1..=50 {
        s1.insert().unwrap();
        s1.set_int("A", i).unwrap();
        s1.set_string("B", &format!("rec{}", i)).unwrap();
    }
    s1.close();

    let s2 = TableScan::new(Rc::clone(&tx), "T", Arc::clone(&layout)).unwrap();
    let c = Constant::from_int(10);
    let t = Term::new(Expression::Field("A".to_string()), Expression::Constant(c));
    let pred = Predicate::from_term(t);
    let s3 = SelectScan::new(Box::new(s2), pred);
    let fields = vec!["B".to_string()];
    let mut s4 = ProjectScan::new(Box::new(s3), fields);
    let mut cnt = 0;
    while s4.next().unwrap() {
        cnt += 1;
        assert_eq!(false, s4.has_field("A"));
        assert_eq!("rec10", s4.get_string("B").unwrap());
    }
    assert_eq!(1, cnt);
}

#[test]
fn scan_test2() {
    let db_dir = ".temp/scantest2";
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new(db_dir).unwrap();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));
    // create tables
    let mut sch1 = Schema::new();
    sch1.add_int_field("A");
    sch1.add_string_field("B", 9);
    let layout1 = Arc::new(Layout::new(Arc::new(sch1)));
    let mut sch2 = Schema::new();
    sch2.add_int_field("C");
    sch2.add_string_field("D", 9);
    let layout2 = Arc::new(Layout::new(Arc::new(sch2)));

    // insert records
    let mut ts1 = TableScan::new(Rc::clone(&tx), "T1", Arc::clone(&layout1)).unwrap();
    for i in 1..=100 {
        ts1.insert().unwrap();
        ts1.set_int("A", i).unwrap();
        ts1.set_string("B", &format!("t1_rec{}", i)).unwrap();
    }
    ts1.close();

    let mut ts2 = TableScan::new(Rc::clone(&tx), "T2", Arc::clone(&layout2)).unwrap();
    for i in 1..=50 {
        ts2.insert().unwrap();
        ts2.set_int("C", i * 2).unwrap();
        ts2.set_string("D", &format!("t2_rec{}", i * 2)).unwrap();
    }
    ts2.close();

    let ts1 = TableScan::new(Rc::clone(&tx), "T1", Arc::clone(&layout1)).unwrap();
    let ts2 = TableScan::new(Rc::clone(&tx), "T2", Arc::clone(&layout2)).unwrap();
    let ts3 = ProductScan::new(Box::new(ts1), Box::new(ts2)).unwrap();
    // selecting all records where A=C
    let t = Term::new(
        Expression::Field("A".to_string()),
        Expression::Field("C".to_string()),
    );
    let pred = Predicate::from_term(t);
    let ts4 = SelectScan::new(Box::new(ts3), pred);

    // projecting on [B,D]
    let fields = vec!["B".to_string(), "D".to_string()];
    let mut ts5 = ProjectScan::new(Box::new(ts4), fields);
    let mut cnt = 0;
    while ts5.next().unwrap() {
        cnt += 1;
        assert_eq!(format!("t1_rec{}", cnt * 2), ts5.get_string("B").unwrap());
        assert_eq!(format!("t2_rec{}", cnt * 2), ts5.get_string("D").unwrap());
    }
    ts5.close();
    tx.borrow_mut().commit().unwrap();
}
