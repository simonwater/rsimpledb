use crate::metadata::MetadataMgr;
use crate::parse::{
    CreateIndexData, CreateTableData, CreateViewData, DeleteData, InsertData, ModifyData,
};
use crate::plan::{Plan, UpdatePlanner};
use crate::plan::{SelectPlan, TablePlan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// The basic planner for SQL update statements
pub struct BasicUpdatePlanner {
    mdm: Arc<MetadataMgr>,
}

impl BasicUpdatePlanner {
    pub fn new(mdm: Arc<MetadataMgr>) -> Self {
        BasicUpdatePlanner { mdm }
    }
}

impl UpdatePlanner for BasicUpdatePlanner {
    fn execute_delete(&self, data: &DeleteData, tx: Rc<RefCell<Transaction>>) -> i32 {
        let mut p: Box<dyn Plan> =
            Box::new(TablePlan::new(Rc::clone(&tx), data.table_name(), &self.mdm));
        if !data.pred().is_empty() {
            p = Box::new(SelectPlan::new(p, data.pred().clone()));
        }
        let mut s = p.open();
        let mut count = 0;
        if let Some(us) = s.as_update_scan() {
            while us.next() {
                us.delete();
                count += 1;
            }
        } else {
            panic!("plan does not produce an UpdateScan");
        }
        s.close();
        count
    }

    fn execute_modify(&self, data: &ModifyData, tx: Rc<RefCell<Transaction>>) -> i32 {
        let mut p: Box<dyn Plan> =
            Box::new(TablePlan::new(Rc::clone(&tx), data.table_name(), &self.mdm));
        if !data.pred().is_empty() {
            p = Box::new(SelectPlan::new(p, data.pred().clone()));
        }
        let mut s = p.open();
        let mut count = 0;
        if let Some(us) = s.as_update_scan() {
            while us.next() {
                let val = data.new_value().evaluate(us);
                us.set_val(data.target_field(), &val);
                count += 1;
            }
        } else {
            panic!("plan does not produce an UpdateScan");
        }
        s.close();
        count
    }

    fn execute_insert(&self, data: &InsertData, tx: Rc<RefCell<Transaction>>) -> i32 {
        let p: Box<dyn Plan> =
            Box::new(TablePlan::new(Rc::clone(&tx), data.table_name(), &self.mdm));
        let mut s = p.open();
        if let Some(us) = s.as_update_scan() {
            us.insert();
            let mut iter = data.vals().iter();
            for fldname in data.fields() {
                if let Some(val) = iter.next() {
                    us.set_val(fldname, val);
                }
            }
        } else {
            panic!("plan does not produce an UpdateScan");
        }
        s.close();
        1
    }

    fn execute_create_table(&self, data: &CreateTableData, tx: Rc<RefCell<Transaction>>) -> i32 {
        self.mdm.create_table(data.table_name(), data.schema(), tx);
        0
    }

    fn execute_create_view(&self, data: &CreateViewData, tx: Rc<RefCell<Transaction>>) -> i32 {
        let viewdef = data.view_def();
        self.mdm.create_view(data.view_name(), &viewdef, tx);
        0
    }

    fn execute_create_index(&self, data: &CreateIndexData, tx: Rc<RefCell<Transaction>>) -> i32 {
        self.mdm
            .create_index(data.index_name(), data.table_name(), data.field_name(), tx);
        0
    }
}
