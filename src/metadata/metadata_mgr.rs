use crate::metadata::{IndexInfo, IndexMgr, StatInfo, StatMgr, TableMgr, ViewMgr};
use crate::record::{Layout, Schema};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The metadata manager
#[derive(Clone)]
pub struct MetadataMgr {
    tbl_mgr: TableMgr,
    view_mgr: ViewMgr,
    stat_mgr: StatMgr,
    idx_mgr: IndexMgr,
}

impl MetadataMgr {
    pub fn new(is_new: bool, tx: Rc<RefCell<Transaction>>) -> Self {
        let tbl_mgr = TableMgr::new(is_new, Rc::clone(&tx));
        let view_mgr = ViewMgr::new(is_new, tbl_mgr.clone(), Rc::clone(&tx));
        let stat_mgr = StatMgr::new(tbl_mgr.clone(), Rc::clone(&tx));
        let idx_mgr = IndexMgr::new(is_new, tbl_mgr.clone(), stat_mgr.clone(), tx);

        MetadataMgr {
            tbl_mgr,
            view_mgr,
            stat_mgr,
            idx_mgr,
        }
    }

    pub fn create_table(&self, tblname: &str, sch: Schema, tx: Rc<RefCell<Transaction>>) {
        self.tbl_mgr.create_table(tx, tblname, sch);
    }

    pub fn get_layout(&self, tblname: &str, tx: Rc<RefCell<Transaction>>) -> Layout {
        self.tbl_mgr.get_layout(tblname, tx)
    }

    pub fn create_view(&self, viewname: &str, viewdef: &str, tx: Rc<RefCell<Transaction>>) {
        self.view_mgr.create_view(viewname, viewdef, tx);
    }

    pub fn get_view_def(&self, viewname: &str, tx: Rc<RefCell<Transaction>>) -> Option<String> {
        self.view_mgr.get_view_def(viewname, tx)
    }

    pub fn create_index(
        &self,
        idxname: &str,
        tblname: &str,
        fldname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) {
        self.idx_mgr.create_index(idxname, tblname, fldname, tx);
    }

    pub fn get_index_info(
        &self,
        tblname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> HashMap<String, IndexInfo> {
        self.idx_mgr.get_index_info(tblname, tx)
    }

    pub fn get_stat_info(
        &self,
        tblname: &str,
        layout: Rc<Layout>,
        tx: Rc<RefCell<Transaction>>,
    ) -> StatInfo {
        self.stat_mgr.get_stat_info(tblname, layout, tx)
    }
}
