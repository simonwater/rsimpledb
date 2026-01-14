use crate::metadata::{MAX_NAME, TableMgr};
use crate::query::{Scan, UpdateScan};
use crate::record::{Schema, TableScan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// The max chars in a view definition
const MAX_VIEWDEF: i32 = 100;

/// The view manager
#[derive(Clone)]
pub struct ViewMgr {
    tbl_mgr: TableMgr,
}

impl ViewMgr {
    pub fn new(is_new: bool, tbl_mgr: TableMgr, tx: Rc<RefCell<Transaction>>) -> Self {
        if is_new {
            let mut sch = Schema::new();
            sch.add_string_field("viewname", MAX_NAME);
            sch.add_string_field("viewdef", MAX_VIEWDEF);
            tbl_mgr.create_table("viewcat", Arc::new(sch), Rc::clone(&tx));
        }
        ViewMgr { tbl_mgr }
    }

    pub fn create_view(&self, vname: &str, vdef: &str, tx: Rc<RefCell<Transaction>>) {
        let layout = self.tbl_mgr.get_layout("viewcat", Rc::clone(&tx));
        let mut ts = TableScan::new(tx, "viewcat", Arc::new(layout));
        ts.insert();
        ts.set_string("viewname", vname);
        ts.set_string("viewdef", vdef);
        ts.close();
    }

    pub fn get_view_def(&self, vname: &str, tx: Rc<RefCell<Transaction>>) -> Option<String> {
        let layout = self.tbl_mgr.get_layout("viewcat", Rc::clone(&tx));
        let mut ts = TableScan::new(tx, "viewcat", Arc::new(layout));
        let mut result = None;
        while ts.next() {
            if ts.get_string("viewname") == vname {
                result = Some(ts.get_string("viewdef"));
                break;
            }
        }
        ts.close();
        result
    }
}
