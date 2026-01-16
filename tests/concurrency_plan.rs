use rsimpledb::DataBase;
use rsimpledb::plan::Planner;
use rsimpledb::thread::MultiThreadRunner;
use rsimpledb::tx::Transaction;
use rsimpledb::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
pub fn planner_multi_thread1() {
    let db_dir = ".temp/concur_plannerdb1";
    let _guard = TempFileGuard::new(db_dir);
    let db = DataBase::new(db_dir).unwrap();
    let planner = db.planner();
    let tx = Rc::new(RefCell::new(db.new_tx()));
    // create table in main thread
    create_table(planner, tx.clone(), "T");
    insert_table(planner, tx.clone(), "T", (0, 1000));
    tx.borrow_mut().commit();

    let headers = vec![];
    let runner = MultiThreadRunner::new(2, headers);
    runner.excute(move |tid| {
        let tx = Rc::new(RefCell::new(db.new_tx()));
        let planner = db.planner();
        update_table(&planner, Rc::clone(&tx), "T", tid);
        //insert_table(planner, tx.clone(), "T", (tid, tid));
        //check_table(planner, tx.clone(), "T", tid);
        tx.borrow_mut().commit();
        vec![]
    });
}

#[test]
pub fn planner_multi_thread2() {
    let db_dir = ".temp/concur_plannerdb2";
    let _guard = TempFileGuard::new(db_dir);
    let db = DataBase::new(db_dir).unwrap();
    // create tables in main thread
    let tx = Rc::new(RefCell::new(db.new_tx()));
    let planner = db.planner();
    let cmd = "create table T1(A1 int, B1 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    let cmd = "create table T2(A2 int, B2 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
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
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    let total = 5;
    for i in start..(start + total) {
        let cmd = &format!("insert into T1(A1, B1) values({i}, 't1_rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    }

    let cmd = "create table T2(A2 int, B2 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    for i in (start + 2)..(start + 2 + total) {
        let cmd = &format!("insert into T2(A2, B2) values({i}, 't2_rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    }

    let qry = "select B1, B2 from T1, T2 where A1 = A2";
    let qp = planner.create_query_plan(qry, Rc::clone(&tx)).unwrap();
    let mut s = qp.open().unwrap();
    let mut cnt = 0;
    while s.next().unwrap() {
        assert_eq!(
            format!("t1_rec{}", start + 2 + cnt),
            s.get_string("b1").unwrap()
        );
        assert_eq!(
            format!("t2_rec{}", start + 2 + cnt),
            s.get_string("b2").unwrap()
        );
        cnt += 1;
    }
    assert_eq!(3, cnt);
    s.close();
    tx.borrow_mut().commit();
}

fn create_table(planner: &Planner, tx: Rc<RefCell<Transaction>>, tbl: &str) {
    let cmd = &format!("create table {tbl}(A int, B varchar(9))");
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
}

fn insert_table(planner: &Planner, tx: Rc<RefCell<Transaction>>, tbl: &str, range: (usize, usize)) {
    for i in range.0..=range.1 {
        let cmd = &format!("insert into {tbl}(A, B) values({i}, 'rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    }
}

fn update_table(planner: &Planner, tx: Rc<RefCell<Transaction>>, tbl: &str, id: usize) {
    let cmd = &format!("update {tbl} set B = 'rec{id}' where A = {id}");
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
}

fn check_table(planner: &Planner, tx: Rc<RefCell<Transaction>>, tbl: &str, id: usize) {
    let qry = format!("select B from {tbl} where A = {id}");
    let qp = planner.create_query_plan(&qry, Rc::clone(&tx)).unwrap();
    let mut s = qp.open().unwrap();
    while s.next().unwrap() {
        assert_eq!(format!("rec{id}"), s.get_string("b").unwrap());
    }
    s.close();
    tx.borrow_mut().commit();
}
