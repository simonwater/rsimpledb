use rsimpledb::db::DataBase;
use rsimpledb::metadata::IndexType;
use rsimpledb::metadata::MetadataMgr;
use rsimpledb::parse::InsertData;
use rsimpledb::plan::Planner;
use rsimpledb::plan::{BasicUpdatePlanner, UpdatePlanner};
use rsimpledb::query::Constant;
use rsimpledb::tx::Transaction;
use rsimpledb::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use std::vec;

#[test]
fn bulk_insert_test() {
    let db_dir = ".temp/bulk_insertdb";
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new_with_conf(db_dir, 1024, 2048, IndexType::BTree).unwrap();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));
    let planner = db.planner();
    let sql = "create table student(sid int, sname varchar(9), majorid int)";
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

    // Verify records via full table scan
    let start_time = Instant::now();
    basic_query(&planner, tx.clone(), search_val);
    println!(
        "Full table scan: Time taken to query sid={search_val}: {:?}",
        start_time.elapsed()
    );
}

fn bulk_insert(mdm: Arc<MetadataMgr>, tx: Rc<RefCell<Transaction>>, cnt: i32) {
    let planner = BasicUpdatePlanner::new(mdm);
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
