use rsimpledb::db::DataBase;
use rsimpledb::metadata::MetadataMgr;
use rsimpledb::plan::{Plan, Planner, TablePlan};
use rsimpledb::query::Constant;
use rsimpledb::tx::Transaction;
use rsimpledb::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

#[test]
fn index_operations_test() {
    let db_dir = ".temp/bulk_indexdb";
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new_with_size(db_dir, 1024, 1024).unwrap();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));
    let planner = db.planner();
    let sql = "create table student(sid int, sname varchar(9), majorid int)";
    planner.execute_update(sql, Rc::clone(&tx)).unwrap();

    let sql = "create index index_student on student(sid)";
    planner.execute_update(sql, Rc::clone(&tx)).unwrap();

    // Insert records
    for i in 1..=100 {
        let sql = format!(
            "insert into student (sid, sname, majorid) values ({}, 'student{}', {})",
            i, i, i
        );
        planner.execute_update(&sql, Rc::clone(&tx)).unwrap();
    }
    println!("Inserted 100 records into student table.");

    let val = 90;
    // Verify records via basic scan
    let start_time = Instant::now();
    basic_query(&planner, tx.clone(), val);
    println!(
        "Basic Query: Time taken to query sid={val}: {:?}",
        start_time.elapsed()
    );

    // Verify records via index scan
    let start_time = Instant::now();
    index_query(&db.md_mgr(), tx.clone(), val);
    println!(
        "B-tree index Query: Time taken to query sid={val}: {:?}",
        start_time.elapsed()
    );
}

fn basic_query(planner: &Planner, tx: Rc<RefCell<Transaction>>, val: i32) {
    let qry = format!("select sid, sname, majorid from student where sid = {val}");
    let plan = planner.create_query_plan(&qry, Rc::clone(&tx)).unwrap();
    let mut s = plan.open().unwrap();
    let mut cnt = 0;
    while s.next().unwrap() {
        assert_eq!(format!("student{}", val), s.get_string("sname").unwrap());
        cnt += 1;
    }
    assert_eq!(1, cnt);
}

fn index_query(mdm: &MetadataMgr, tx: Rc<RefCell<Transaction>>, val: i32) {
    let student_plan = TablePlan::new(Rc::clone(&tx), "student", mdm).unwrap();
    let mut ts = student_plan.open().unwrap();
    let student_scan = ts.as_update_scan().unwrap();
    let indexes = mdm.get_index_info("student", Rc::clone(&tx)).unwrap();
    let ii = indexes.get("sid").unwrap();
    let mut index_scan = ii.open().unwrap();
    index_scan.before_first(&Constant::from_int(val)).unwrap();
    let mut cnt = 0;
    while index_scan.next().unwrap() {
        let rid = index_scan.get_data_rid().unwrap();
        student_scan.move_to_rid(&rid).unwrap();
        assert_eq!(val, student_scan.get_int("sid").unwrap());
        assert_eq!(
            format!("student{}", val),
            student_scan.get_string("sname").unwrap()
        );
        cnt += 1;
    }
    assert_eq!(1, cnt);
}
