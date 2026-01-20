use crate::DbResult;
use crate::metadata::IndexType;
use crate::metadata::{IndexInfo, MAX_NAME, StatMgr, TableMgr};
use crate::query::{Scan, UpdateScan};
use crate::record::{Layout, Schema, TableScan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// The index manager
#[derive(Clone)]
pub struct IndexMgr {
    layout: Arc<Layout>,
    tbl_mgr: TableMgr,
    stat_mgr: StatMgr,
    index_type: IndexType,
}

impl IndexMgr {
    /// Create the index manager
    pub fn new(
        is_new: bool,
        tbl_mgr: TableMgr,
        stat_mgr: StatMgr,
        tx: Rc<RefCell<Transaction>>,
        index_type: IndexType,
    ) -> DbResult<Self> {
        if is_new {
            let mut sch = Schema::new();
            sch.add_string_field("indexname", MAX_NAME);
            sch.add_string_field("tablename", MAX_NAME);
            sch.add_string_field("fieldname", MAX_NAME);
            tbl_mgr.create_table("idxcat", Arc::new(sch), Rc::clone(&tx))?;
        }
        let layout = tbl_mgr.get_layout("idxcat", tx)?;
        Ok(IndexMgr {
            layout: Arc::new(layout),
            tbl_mgr,
            stat_mgr,
            index_type,
        })
    }

    /// Create an index of the specified type for the specified field
    pub fn create_index(
        &self,
        idxname: &str,
        tblname: &str,
        fldname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<()> {
        let mut ts = TableScan::new(tx, "idxcat", self.layout.clone())?;
        ts.insert()?;
        ts.set_string("indexname", idxname)?;
        ts.set_string("tablename", tblname)?;
        ts.set_string("fieldname", fldname)?;
        ts.close();
        Ok(())
    }

    /// Return a map containing the index info for all indexes on the specified table
    pub fn get_index_info(
        &self,
        tblname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<HashMap<String, IndexInfo>> {
        let mut result = HashMap::new();
        let mut ts = TableScan::new(Rc::clone(&tx), "idxcat", self.layout.clone())?;
        while ts.next()? {
            if ts.get_string("tablename")? == tblname {
                let idxname = ts.get_string("indexname")?;
                let fldname = ts.get_string("fieldname")?;
                let tbl_layout = self.tbl_mgr.get_layout(tblname, Rc::clone(&tx))?;
                let tbl_schema = tbl_layout.schema();
                let tbl_si =
                    self.stat_mgr
                        .get_stat_info(tblname, Arc::new(tbl_layout), Rc::clone(&tx))?;
                let ii = IndexInfo::new(
                    idxname,
                    fldname.clone(),
                    tbl_schema,
                    Rc::clone(&tx),
                    tbl_si,
                    self.index_type.clone(),
                );
                result.insert(fldname, ii);
            }
        }
        ts.close();
        Ok(result)
    }
}
