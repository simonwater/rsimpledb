use crate::file::BlockId;
use crate::tx::concurrency::{LockAbortException, LockTable};
use std::collections::HashMap;

/// The concurrency manager for the transaction
pub struct ConcurrencyMgr {
    locktbl: LockTable,
    locks: HashMap<BlockId, String>,
}

impl ConcurrencyMgr {
    pub fn new(locktbl: LockTable) -> Self {
        ConcurrencyMgr {
            locktbl,
            locks: HashMap::new(),
        }
    }

    /// Obtain an SLock on the block, if necessary
    pub fn s_lock(&mut self, blk: &BlockId) -> Result<(), LockAbortException> {
        if !self.locks.contains_key(blk) {
            self.locktbl.s_lock(blk)?;
            self.locks.insert(blk.clone(), "S".to_string());
        }
        Ok(())
    }

    /// Obtain an XLock on the block, if necessary
    pub fn x_lock(&mut self, blk: &BlockId) -> Result<(), LockAbortException> {
        if !self.has_x_lock(blk) {
            self.s_lock(blk)?;
            self.locktbl.x_lock(blk)?;
            self.locks.insert(blk.clone(), "X".to_string());
        }
        Ok(())
    }

    /// Release all locks
    pub fn release(&mut self) {
        for blk in self.locks.keys() {
            self.locktbl.unlock(blk);
        }
        self.locks.clear();
    }

    fn has_x_lock(&self, blk: &BlockId) -> bool {
        self.locks
            .get(blk)
            .map(|locktype| locktype == "X")
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use crate::DataBase;
    use crate::file::BlockId;
    use crate::tx::Transaction;
    use crate::util::TempFileGuard;

    #[test]
    fn concurrency_mgr_test() {
        let db_dir = ".temp/concurrencydb";
        let _guard = TempFileGuard::new(db_dir);
        let db = DataBase::new(db_dir);
        let mut fm = db.file_mgr();
        let _blk1 = fm.append("testfile");
        let _blk2 = fm.append("testfile");

        let mut handlers = vec![];
        let tx_a = db.new_tx();
        let t1 = thread::spawn(move || {
            run_a(tx_a);
        });
        handlers.push(t1);

        let tx_b = db.new_tx();
        let t2 = thread::spawn(move || {
            run_b(tx_b);
        });
        handlers.push(t2);

        let tx_c = db.new_tx();
        let t3 = thread::spawn(move || {
            run_c(tx_c);
        });
        handlers.push(t3);

        for handle in handlers {
            handle.join().unwrap();
        }
    }

    fn run_a(mut tx_a: Transaction) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        tx_a.pin(&blk1).unwrap();
        tx_a.pin(&blk2).unwrap();
        println!("Tx A: request slock 1");
        tx_a.get_int(&blk1, 0).unwrap();
        println!("Tx A: receive slock 1");
        thread::sleep(Duration::from_millis(1000));
        println!("Tx A: request slock 2");
        tx_a.get_int(&blk2, 0).unwrap();
        println!("Tx A: receive slock 2");
        tx_a.commit();
        println!("Tx A: commit");
    }

    fn run_b(mut tx_b: Transaction) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        tx_b.pin(&blk1).unwrap();
        tx_b.pin(&blk2).unwrap();
        println!("Tx B: request xlock 2");
        tx_b.set_int(&blk2, 0, 0, false).unwrap();
        println!("Tx B: receive xlock 2");
        thread::sleep(Duration::from_millis(1000));
        println!("Tx B: request slock 1");
        tx_b.get_int(&blk1, 0).unwrap();
        println!("Tx B: receive slock 1");
        tx_b.commit();
        println!("Tx B: commit");
    }

    fn run_c(mut tx_c: Transaction) {
        let blk1 = BlockId::new("testfile".to_string(), 0);
        let blk2 = BlockId::new("testfile".to_string(), 1);
        tx_c.pin(&blk1).unwrap();
        tx_c.pin(&blk2).unwrap();
        thread::sleep(Duration::from_millis(500));
        println!("Tx C: request xlock 1");
        tx_c.set_int(&blk1, 0, 0, false).unwrap();
        println!("Tx C: receive xlock 1");
        thread::sleep(Duration::from_millis(1000));
        println!("Tx C: request slock 2");
        tx_c.get_int(&blk2, 0).unwrap();
        println!("Tx C: receive slock 2");
        tx_c.commit();
        println!("Tx C: commit");
    }
}
