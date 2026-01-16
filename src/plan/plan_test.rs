use crate::DataBase;
use crate::util::TempFileGuard;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn single_table_test() {
    let db_dir = ".temp/singletableplanner";
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new(db_dir).unwrap();
    let planner = db.planner();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));

    let cmd = "create table T1(A int, B varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();

    for i in 1..=50 {
        let cmd = &format!("insert into T1(A,B) values({i}, 'rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    }

    let qry = "select B from T1 where A = 10";
    let qp = planner.create_query_plan(qry, Rc::clone(&tx)).unwrap();
    let mut s = qp.open().unwrap();
    while s.next().unwrap() {
        assert_eq!("rec10", s.get_string("b").unwrap());
    }
    s.close();
    tx.borrow_mut().commit().unwrap();
}

#[test]
fn multi_table_test() {
    let db_dir = ".temp/multitableplanner";
    let _guard = TempFileGuard::new(db_dir);
    let db: DataBase = DataBase::new(db_dir).unwrap();
    let planner = db.planner();
    let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));

    let cmd = "create table T1(A1 int, B1 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    for i in 1..=100 {
        let cmd = &format!("insert into T1(A1, B1) values({i}, 't1_rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    }

    let cmd = "create table T2(A2 int, B2 varchar(9))";
    planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    for i in 51..=150 {
        let cmd = &format!("insert into T2(A2, B2) values({i}, 't2_rec{i}')");
        planner.execute_update(cmd, Rc::clone(&tx)).unwrap();
    }

    let qry = "select B1, B2 from T1, T2 where A1 = A2";
    let qp = planner.create_query_plan(qry, Rc::clone(&tx)).unwrap();
    let mut s = qp.open().unwrap();
    let mut cnt = 0;
    while s.next().unwrap() {
        cnt += 1;
        assert_eq!(format!("t1_rec{}", cnt + 50), s.get_string("b1").unwrap());
        assert_eq!(format!("t2_rec{}", cnt + 50), s.get_string("b2").unwrap());
    }
    assert_eq!(50, cnt);
    s.close();
    tx.borrow_mut().commit().unwrap();
}
