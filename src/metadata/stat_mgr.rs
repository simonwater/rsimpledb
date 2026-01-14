use crate::metadata::{StatInfo, TableMgr};
use crate::query::{Scan, UpdateScan};
use crate::record::{Layout, TableScan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// The statistics manager
#[derive(Clone)]
pub struct StatMgr {
    tbl_mgr: TableMgr,
    table_stats: Arc<Mutex<HashMap<String, StatInfo>>>,
    num_calls: Arc<Mutex<i32>>,
}

impl StatMgr {
    /// Create the statistics manager
    pub fn new(tbl_mgr: TableMgr, tx: Rc<RefCell<Transaction>>) -> Self {
        let stat_mgr = StatMgr {
            tbl_mgr,
            table_stats: Arc::new(Mutex::new(HashMap::new())),
            num_calls: Arc::new(Mutex::new(0)),
        };
        stat_mgr.refresh_statistics(tx);
        stat_mgr
    }

    /// Return the statistical information about the specified table
    pub fn get_stat_info(
        &self,
        tblname: &str,
        layout: Arc<Layout>,
        tx: Rc<RefCell<Transaction>>,
    ) -> StatInfo {
        {
            let mut num_calls = self.num_calls.lock().unwrap();
            *num_calls += 1;
            if *num_calls > 100 {
                drop(num_calls);
                self.refresh_statistics(Rc::clone(&tx));
            }
        }

        {
            let table_stats = self.table_stats.lock().unwrap();
            if let Some(si) = table_stats.get(tblname) {
                return si.clone();
            }
        }

        let si = self.calc_table_stats(tblname, layout, Rc::clone(&tx));
        {
            let mut table_stats = self.table_stats.lock().unwrap();
            table_stats.insert(tblname.to_string(), si.clone());
        }
        si
    }

    fn refresh_statistics(&self, tx: Rc<RefCell<Transaction>>) {
        {
            let mut table_stats = self.table_stats.lock().unwrap();
            table_stats.clear();
        }
        {
            let mut num_calls = self.num_calls.lock().unwrap();
            *num_calls = 0;
        }
        let tcat_layout = self.tbl_mgr.get_layout("tblcat", Rc::clone(&tx));
        let mut tcat = TableScan::new(Rc::clone(&tx), "tblcat", Arc::new(tcat_layout));
        while tcat.next() {
            let tblname = tcat.get_string("tblname");
            let layout = self.tbl_mgr.get_layout(&tblname, Rc::clone(&tx));
            let si = self.calc_table_stats(&tblname, Arc::new(layout), Rc::clone(&tx));
            let mut table_stats = self.table_stats.lock().unwrap();
            table_stats.insert(tblname, si);
        }
        tcat.close();
    }

    fn calc_table_stats(
        &self,
        tblname: &str,
        layout: Arc<Layout>,
        tx: Rc<RefCell<Transaction>>,
    ) -> StatInfo {
        let mut num_recs = 0;
        let mut num_blocks = 0;
        let mut ts = TableScan::new(tx, tblname, layout);
        while ts.next() {
            num_recs += 1;
            num_blocks = ts.get_rid().block_number() + 1;
        }
        ts.close();
        StatInfo::new(num_blocks, num_recs)
    }
}
