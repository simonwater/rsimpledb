use std::sync::Arc;
use crate::plan::Plan;
use crate::query::Scan;
use crate::record::{Schema, Layout, TableScan};
use crate::metadata::MetadataMgr;
use crate::tx::Transaction;

/// The Plan class corresponding to a table
pub struct TablePlan {
    tblname: String,
    tx: Arc<std::sync::Mutex<Transaction>>,
    layout: Layout,
    si: crate::metadata::StatInfo,
}

impl TablePlan {
    /// Creates a leaf node in the query tree corresponding to the specified table
    pub fn new(tx: Arc<std::sync::Mutex<Transaction>>, tblname: &str, md: &MetadataMgr) -> Self {
        let layout = md.get_layout(tblname, Arc::clone(&tx));
        let si = md.get_stat_info(tblname, &layout, Arc::clone(&tx));
        TablePlan {
            tblname: tblname.to_string(),
            tx,
            layout,
            si,
        }
    }
}

impl Plan for TablePlan {
    /// Creates a table scan for this query
    fn open(&self) -> Box<dyn Scan> {
        Box::new(TableScan::new(Arc::clone(&self.tx), &self.tblname, self.layout.clone()))
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
    fn schema(&self) -> Schema {
        self.layout.schema().clone()
    }
}

