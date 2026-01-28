use rsimpledb::db::DataBase;
use rsimpledb::index::planner::IndexUpdatePlanner;
use rsimpledb::metadata::IndexType;
use rsimpledb::metadata::MetadataMgr;
use rsimpledb::parse::InsertData;
use rsimpledb::plan::UpdatePlanner;
use rsimpledb::plan::{Plan, Planner, TablePlan};
use rsimpledb::query::Constant;
use rsimpledb::tx::Transaction;
use rsimpledb::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use std::vec;

#[test]
fn btree_bulk_index_test() {
    index_operation_test(".temp/btree_bulk_indexdb", IndexType::BTree);
}

#[test]
fn hash_bulk_index_test() {
    index_operation_test(".temp/hash_bulk_indexdb", IndexType::Hash);
}

fn index_operation_test(db_dir: &str, index_type: IndexType) {
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new_with_conf(db_dir, 4096, 2048, index_type.clone()).unwrap();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));
    let planner = db.planner();
    let sql = "create table student(sid int, sname varchar(9), majorid int)";
    planner.execute_update(sql, Rc::clone(&tx)).unwrap();

    let sql = "create index index_student on student(sid)";
    planner.execute_update(sql, Rc::clone(&tx)).unwrap();

    // Insert records
    let total = 10000;
    let start_time = Instant::now();
    bulk_insert(db.md_mgr(), Rc::clone(&tx), total);
    println!(
        "Inserted {total} records into student table. Time taken: {:?}",
        start_time.elapsed()
    );

    let search_val = 5432;

    // Verify records via index scan
    let start_time = Instant::now();
    index_query(db.md_mgr(), tx.clone(), search_val);
    println!(
        "{} index Query: Time taken to query sid={search_val}: {:?}",
        &index_type,
        start_time.elapsed()
    );

    // Verify records via full table scan
    let start_time = Instant::now();
    basic_query(&planner, tx.clone(), search_val);
    println!(
        "Full table scan: Time taken to query sid={search_val}: {:?}",
        start_time.elapsed()
    );
}

fn bulk_insert(mdm: Arc<MetadataMgr>, tx: Rc<RefCell<Transaction>>, cnt: i32) {
    let planner = IndexUpdatePlanner::new(mdm);
    let mut rows = vec![];
    for i in 1..=cnt {
        let row = vec![
            Constant::from_int(i),
            Constant::from_string(format!("student{}", i)),
            Constant::from_int(i),
        ];
        rows.push(row);
    }
    let insert_data = InsertData::new(
        "student".to_string(),
        vec![
            "sid".to_string(),
            "sname".to_string(),
            "majorid".to_string(),
        ],
        rows,
    );
    planner
        .execute_insert(&insert_data, Rc::clone(&tx))
        .unwrap();
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

fn index_query(mdm: Arc<MetadataMgr>, tx: Rc<RefCell<Transaction>>, val: i32) {
    let student_plan = TablePlan::new(Rc::clone(&tx), "student", &mdm).unwrap();
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
