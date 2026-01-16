use crate::DbResult;
use crate::plan::Plan;
use crate::query::{Predicate, Scan, SelectScan};
use crate::record::Schema;
use std::sync::Arc;

/// The Plan class corresponding to the select relational algebra operator
pub struct SelectPlan {
    p: Box<dyn Plan>,
    pred: Predicate,
}

impl SelectPlan {
    /// Creates a new select node in the query tree
    pub fn new(p: Box<dyn Plan>, pred: Predicate) -> Self {
        SelectPlan { p, pred }
    }
}

impl Plan for SelectPlan {
    /// Creates a select scan for this query
    fn open(&self) -> DbResult<Box<dyn Scan>> {
        let s = self.p.open()?;
        Ok(Box::new(SelectScan::new(s, self.pred.clone())))
    }

    /// Estimates the number of block accesses in the selection
    fn blocks_accessed(&self) -> i32 {
        self.p.blocks_accessed()
    }

    /// Estimates the number of output records in the selection
    fn records_output(&self) -> i32 {
        self.p.records_output() / self.pred.reduction_factor(self.p.as_ref())
    }

    /// Estimates the number of distinct field values
    fn distinct_values(&self, fldname: &str) -> i32 {
        if let Some(_) = self.pred.equates_with_constant(fldname) {
            1
        } else if let Some(fldname2) = self.pred.equates_with_field(fldname) {
            std::cmp::min(
                self.p.distinct_values(fldname),
                self.p.distinct_values(&fldname2),
            )
        } else {
            self.p.distinct_values(fldname)
        }
    }

    /// Returns the schema of the selection
    fn schema(&self) -> Arc<Schema> {
        self.p.schema()
    }
}
