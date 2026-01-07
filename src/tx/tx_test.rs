#[cfg(test)]
pub mod tests {

    use crate::DataBase;
    use crate::file::BlockId;
    use crate::thread::MultiThreadRunner;

    #[test]
    fn tx_single_thread() {
        let db: DataBase = DataBase::new(".temp/txdb1");
        let mut fm = db.file_mgr();
        let blk = fm.append("testfile");
        tx_test(db, blk);
    }

    #[test]
    pub fn tx_multi_thread() {
        let db = DataBase::new(".temp/txdb2");
        let headers = vec![];
        let runner = MultiThreadRunner::new(100, headers);
        runner.excute(move |_tid| {
            let db = db.clone();
            let mut fm = db.file_mgr();
            let blk = fm.append("testfile");
            tx_test(db, blk);
            vec![]
        });
    }

    fn tx_test(db: DataBase, blk: BlockId) {
        let mut tx1 = db.new_tx();
        tx1.pin(&blk).unwrap();
        tx1.set_int(&blk, 0, 123, true).unwrap();
        tx1.set_string(&blk, 10, "hello", true).unwrap();
        assert_eq!(123, tx1.get_int(&blk, 0).unwrap());
        assert_eq!("hello".to_string(), tx1.get_string(&blk, 10).unwrap());
        tx1.commit();

        let mut tx2 = db.new_tx();
        tx2.pin(&blk).unwrap();
        let ival = tx2.get_int(&blk, 0).unwrap();
        let sval = tx2.get_string(&blk, 10).unwrap();
        assert_eq!(123, ival);
        assert_eq!("hello".to_string(), sval);
        tx2.set_int(&blk, 0, 456, true).unwrap();
        tx2.set_string(&blk, 10, "world", true).unwrap();
        assert_eq!(456, tx2.get_int(&blk, 0).unwrap());
        assert_eq!("world".to_string(), tx2.get_string(&blk, 10).unwrap());
        tx2.commit();

        let mut tx3 = db.new_tx();
        tx3.pin(&blk).unwrap();
        assert_eq!(456, tx3.get_int(&blk, 0).unwrap());
        assert_eq!("world".to_string(), tx3.get_string(&blk, 10).unwrap());
        tx3.set_int(&blk, 0, 999, true).unwrap();
        assert_eq!(999, tx3.get_int(&blk, 0).unwrap());
        tx3.rollback();

        let mut tx4 = db.new_tx();
        tx4.pin(&blk).unwrap();
        assert_eq!(456, tx4.get_int(&blk, 0).unwrap());
        assert_eq!("world".to_string(), tx4.get_string(&blk, 10).unwrap());
        tx4.commit();
    }
}
