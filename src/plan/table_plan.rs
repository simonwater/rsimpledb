use crate::DbResult;
use crate::metadata::MetadataMgr;
use crate::plan::Plan;
use crate::query::Scan;
use crate::record::{Layout, Schema, TableScan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// The Plan class corresponding to a table
pub struct TablePlan {
    tblname: String,
    tx: Rc<RefCell<Transaction>>,
    layout: Arc<Layout>,
    si: crate::metadata::StatInfo,
}

impl TablePlan {
    /// Creates a leaf node in the query tree corresponding to the specified table
    pub fn new(tx: Rc<RefCell<Transaction>>, tblname: &str, md: &MetadataMgr) -> DbResult<Self> {
        let layout = md.get_layout(tblname, Rc::clone(&tx))?;
        let arc_layout = Arc::new(layout);
        let si = md.get_stat_info(tblname, Arc::clone(&arc_layout), Rc::clone(&tx))?;
        Ok(TablePlan {
            tblname: tblname.to_string(),
            tx,
            layout: arc_layout,
            si,
        })
    }
}

impl Plan for TablePlan {
    /// Creates a table scan for this query
    fn open(&self) -> DbResult<Box<dyn Scan>> {
        Ok(Box::new(TableScan::new(
            Rc::clone(&self.tx),
            &self.tblname,
            self.layout.clone(),
        )?))
    }

    /// Estimates the number of block accesses for the table
    fn blocks_accessed(&self) -> i32 {
        self.si.blocks_accessed()
    }

    /// Estimates the number of records in the table
    fn records_output(&self) -> i32 {
        self.si.records_output()
    }

    /// Estimates the number of distinct field values in the table
    fn distinct_values(&self, fldname: &str) -> i32 {
        self.si.distinct_values(fldname)
    }

    /// Determines the schema of the table
    fn schema(&self) -> Arc<Schema> {
        self.layout.schema()
    }
}
