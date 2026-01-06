use crate::buffer::BufferMgr;
use crate::file::FileMgr;
use crate::log::LogMgr;
use crate::tx::{LockTable, Transaction};
use std::path::PathBuf;

pub const BLOCK_SIZE: usize = 400;
pub const BUFFER_SIZE: usize = 8;
pub const LOG_FILE: &'static str = "rsimpledb.log";

pub struct DataBase {
    lm: LogMgr,
    bm: BufferMgr,
    fm: FileMgr,
    lt: LockTable,
}

impl DataBase {
    pub fn new(db_directory: &str) -> Self {
        Self::new_with_size(db_directory, BUFFER_SIZE, BLOCK_SIZE)
    }

    pub fn new_with_size(db_directory: &str, numbuffs: usize, blocksize: usize) -> Self {
        let db_dir = PathBuf::from(db_directory);
        let fm = FileMgr::new(db_dir, blocksize);
        let lm = LogMgr::new(fm.clone(), LOG_FILE.to_string());
        let bm = BufferMgr::new(fm.clone(), lm.clone(), numbuffs);
        let lt = LockTable::new();

        let mut tx = Transaction::new(fm.clone(), lm.clone(), bm.clone(), lt.clone());
        let isnew = fm.is_new();
        if isnew {
            println!("creating new database");
        } else {
            println!("recovering existing database");
            tx.recover();
        }
        tx.commit();

        DataBase { lm, bm, fm, lt }
    }

    pub fn file_mgr(&mut self) -> &mut FileMgr {
        &mut self.fm
    }

    pub fn log_mgr(&mut self) -> &mut LogMgr {
        &mut self.lm
    }

    pub fn buffer_mgr(&mut self) -> &mut BufferMgr {
        &mut self.bm
    }

    pub fn new_tx(&self) -> Transaction {
        Transaction::new(
            self.fm.clone(),
            self.lm.clone(),
            self.bm.clone(),
            self.lt.clone(),
        )
    }
}
