use crate::db::DataBase;
use crate::metadata::{IndexType, MetadataMgr};
use crate::plan::{Plan, TablePlan};
use crate::query::Constant;
use crate::tx::Transaction;
use crate::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn btree_index_test() {
    index_operation_test(".temp/btree_indexdb", IndexType::BTree);
}

#[test]
fn hash_index_test() {
    index_operation_test(".temp/hash_indexdb", IndexType::Hash);
}

fn index_operation_test(db_dir: &str, index_type: IndexType) {
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new_with_conf(db_dir, 1024, 1024, index_type).unwrap();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));
    let planner = db.planner();
    let sql = "create table student(sid int, sname varchar(9), majorid int)";
    planner.execute_update(sql, Rc::clone(&tx)).unwrap();

    let sql = "create index index_student on student(sid)";
    planner.execute_update(sql, Rc::clone(&tx)).unwrap();

    // Insert records
    for i in 1..=100 {
        let sql =
            format!("insert into student (sid, sname, majorid) values ({i}, 'student{i}', {i})");
        planner.execute_update(&sql, Rc::clone(&tx)).unwrap();
    }

    // Verify records via index scan
    index_query(&db.md_mgr(), tx.clone(), "sid", 90);
    tx.borrow_mut().commit().unwrap();
}

fn index_query(mdm: &MetadataMgr, tx: Rc<RefCell<Transaction>>, key: &str, val: i32) {
    let student_plan = TablePlan::new(Rc::clone(&tx), "student", mdm).unwrap();
    let mut ts = student_plan.open().unwrap();
    let student_scan = ts.as_update_scan().unwrap();
    let mut indexes = mdm.get_index_info("student", Rc::clone(&tx)).unwrap();
    let ii = indexes.get_mut(key).unwrap();
    let mut index_scan = ii.open().unwrap();
    index_scan.before_first(&Constant::from_int(val)).unwrap();
    let mut cnt = 0;
    while index_scan.next().unwrap() {
        let rid = index_scan.get_data_rid().unwrap();
        student_scan.move_to_rid(&rid).unwrap();
        assert_eq!(
            format!("student{}", val),
            student_scan.get_string("sname").unwrap()
        );
        cnt += 1;
    }
    assert_eq!(1, cnt);
}
