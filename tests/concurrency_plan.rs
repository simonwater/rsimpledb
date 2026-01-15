use rsimpledb::DataBase;
use rsimpledb::thread::MultiThreadRunner;
use rsimpledb::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
pub fn planner_multi_thread1() {
    let db_dir = ".temp/concur_plannerdb1";
    let _guard = TempFileGuard::new(db_dir);
    let db = DataBase::new(db_dir);
    // create table in main thread
    let tx = Rc::new(RefCell::new(db.new_tx()));
    let planner = db.planner();
    let cmd = "create table T(A int, B varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx));
    tx.borrow_mut().commit();

    let headers = vec![];
    let runner = MultiThreadRunner::new(4, headers);
    runner.excute(move |tid| {
        single_table_test(&db, tid);
        vec![]
    });
}

fn single_table_test(db: &DataBase, tid: usize) {
    let planner = db.planner();
    let tx = Rc::new(RefCell::new(db.new_tx()));

    let cmd = "create table T(A int, B varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx));

    let start = tid * 100;
    let total = 5;
    for i in start..(start + total) {
        let cmd = &format!("insert into T(A, B) values({i}, 'rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx));
    }

    let qry = "select B from T where A = 10";
    let qp = planner.create_query_plan(qry, Rc::clone(&tx));
    let mut s = qp.open();
    while s.next() {
        assert_eq!("rec10", s.get_string("b"));
    }
    s.close();
    tx.borrow_mut().commit();
}

#[test]
pub fn planner_multi_thread2() {
    let db_dir = ".temp/concur_plannerdb2";
    let _guard = TempFileGuard::new(db_dir);
    let db = DataBase::new(db_dir);
    // create tables in main thread
    let tx = Rc::new(RefCell::new(db.new_tx()));
    let planner = db.planner();
    let cmd = "create table T1(A1 int, B1 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx));
    let cmd = "create table T2(A2 int, B2 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx));
    tx.borrow_mut().commit();

    let headers = vec![];
    let runner = MultiThreadRunner::new(10, headers);
    runner.excute(move |tid| {
        multi_table_test(&db, tid);
        vec![]
    });
}

fn multi_table_test(db: &DataBase, tid: usize) {
    let planner = db.planner();
    let tx = Rc::new(RefCell::new(db.new_tx()));

    let start = tid * 100;
    let cmd = "create table T1(A1 int, B1 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx));
    let total = 5;
    for i in start..(start + total) {
        let cmd = &format!("insert into T1(A1, B1) values({i}, 't1_rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx));
    }

    let cmd = "create table T2(A2 int, B2 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx));
    for i in (start + 2)..(start + 2 + total) {
        let cmd = &format!("insert into T2(A2, B2) values({i}, 't2_rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx));
    }

    let qry = "select B1, B2 from T1, T2 where A1 = A2";
    let qp = planner.create_query_plan(qry, Rc::clone(&tx));
    let mut s = qp.open();
    let mut cnt = 0;
    while s.next() {
        assert_eq!(format!("t1_rec{}", start + 2 + cnt), s.get_string("b1"));
        assert_eq!(format!("t2_rec{}", start + 2 + cnt), s.get_string("b2"));
        cnt += 1;
    }
    assert_eq!(3, cnt);
    s.close();
    tx.borrow_mut().commit();
}
