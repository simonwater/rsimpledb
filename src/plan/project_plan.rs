use crate::DbResult;
use crate::plan::Plan;
use crate::query::{ProjectScan, Scan};
use crate::record::Schema;
use std::sync::Arc;

/// The Plan class corresponding to the project relational algebra operator
pub struct ProjectPlan {
    p: Box<dyn Plan>,
    schema: Arc<Schema>,
}

impl ProjectPlan {
    /// Creates a new project node in the query tree
    pub fn new(p: Box<dyn Plan>, fieldlist: Vec<String>) -> Self {
        let mut schema = crate::record::Schema::new();
        for fldname in &fieldlist {
            schema.add(fldname, &p.schema());
        }
        ProjectPlan {
            p,
            schema: Arc::new(schema),
        }
    }
}

impl Plan for ProjectPlan {
    /// Creates a project scan for this query
    fn open(&self) -> DbResult<Box<dyn Scan>> {
        let s = self.p.open()?;
        Ok(Box::new(ProjectScan::new(s, self.schema.fields().to_vec())))
    }

    /// Estimates the number of block accesses in the projection
    fn blocks_accessed(&self) -> i32 {
        self.p.blocks_accessed()
    }

    /// Estimates the number of output records in the projection
    fn records_output(&self) -> i32 {
        self.p.records_output()
    }

    /// Estimates the number of distinct field values in the projection
    fn distinct_values(&self, fldname: &str) -> i32 {
        self.p.distinct_values(fldname)
    }

    /// Returns the schema of the projection
    fn schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
}
