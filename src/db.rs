use crate::DbResult;
use crate::buffer::BufferMgr;
use crate::file::FileMgr;
use crate::index::planner::IndexUpdatePlanner;
use crate::log::LogMgr;
use crate::metadata::MetadataMgr;
use crate::plan::{BasicQueryPlanner, BasicUpdatePlanner, Planner};
use crate::tx::{LockTable, Transaction};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

pub const BLOCK_SIZE: usize = 1024;
pub const BUFFER_SIZE: usize = 100;
pub const LOG_FILE: &'static str = "rsimpledb.log";

#[derive(Clone)]
pub struct DataBase {
    lm: LogMgr,
    bm: BufferMgr,
    fm: FileMgr,
    lt: LockTable,
    mdm: Arc<MetadataMgr>,
    planner: Planner,
}

impl DataBase {
    pub fn new(db_directory: &str) -> DbResult<Self> {
        Self::new_with_size(db_directory, BLOCK_SIZE, BUFFER_SIZE)
    }

    pub fn new_with_size(db_directory: &str, blocksize: usize, numbuffs: usize) -> DbResult<Self> {
        let db_dir = PathBuf::from(db_directory);
        let fm = FileMgr::new(db_dir, blocksize)?;
        let lm = LogMgr::new(fm.clone(), LOG_FILE.to_string())?;
        let bm = BufferMgr::new(fm.clone(), lm.clone(), numbuffs);
        let lt = LockTable::new();

        let tx = Transaction::new(fm.clone(), lm.clone(), bm.clone(), lt.clone())?;
        let tx = Rc::new(RefCell::new(tx));
        let isnew = fm.is_new();
        let mdm = Arc::new(MetadataMgr::new(isnew, Rc::clone(&tx))?);
        if isnew {
            println!("creating new database");
        } else {
            println!("recovering existing database");
            tx.borrow_mut().recover()?;
        }
        let qp = Arc::new(BasicQueryPlanner::new(Arc::clone(&mdm)));
        let _up = Arc::new(BasicUpdatePlanner::new(Arc::clone(&mdm)));

        let _up = Arc::new(IndexUpdatePlanner::new(Arc::clone(&mdm)));
        let planner = Planner::new(qp, _up);
        tx.borrow_mut().commit()?;

        Ok(DataBase {
            lm,
            bm,
            fm,
            lt,
            mdm,
            planner,
        })
    }

    pub fn file_mgr(&self) -> FileMgr {
        self.fm.clone()
    }

    pub fn log_mgr(&self) -> LogMgr {
        self.lm.clone()
    }

    pub fn buffer_mgr(&self) -> BufferMgr {
        self.bm.clone()
    }

    pub fn md_mgr(&self) -> Arc<MetadataMgr> {
        self.mdm.clone()
    }

    pub fn planner(&self) -> Planner {
        self.planner.clone()
    }

    pub fn new_tx(&self) -> DbResult<Transaction> {
        Transaction::new(
            self.fm.clone(),
            self.lm.clone(),
            self.bm.clone(),
            self.lt.clone(),
        )
    }
}
