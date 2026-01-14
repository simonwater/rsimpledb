use std::sync::Arc;
use crate::plan::{Plan, QueryPlanner};
use crate::metadata::MetadataMgr;
use crate::parse::{QueryData, Parser};
use crate::plan::{TablePlan, ProductPlan, SelectPlan, ProjectPlan};
use crate::tx::Transaction;

/// The simplest, most naive query planner possible
pub struct BasicQueryPlanner {
    mdm: MetadataMgr,
}

impl BasicQueryPlanner {
    pub fn new(mdm: MetadataMgr) -> Self {
        BasicQueryPlanner { mdm }
    }
}

impl QueryPlanner for BasicQueryPlanner {
    /// Creates a query plan as follows:
    /// 1. Create a plan for each mentioned table or view
    /// 2. Create the product of all table plans
    /// 3. Add a selection plan for the predicate
    /// 4. Project on the field names
    fn create_plan(&self, data: &QueryData, tx: Arc<std::sync::Mutex<Transaction>>) -> Box<dyn Plan> {
        // Step 1: Create a plan for each mentioned table or view
        let mut plans: Vec<Box<dyn Plan>> = Vec::new();
        for tblname in data.tables() {
            let viewdef = self.mdm.get_view_def(tblname, Arc::clone(&tx));
            if let Some(viewdef_str) = viewdef {
                // Recursively plan the view
                let mut parser = Parser::new(&viewdef_str);
                let viewdata = parser.query();
                plans.push(self.create_plan(&viewdata, Arc::clone(&tx)));
            } else {
                plans.push(Box::new(TablePlan::new(Arc::clone(&tx), tblname, &self.mdm)));
            }
        }

        // Step 2: Create the product of all table plans
        let mut p = plans.remove(0);
        for nextplan in plans {
            p = Box::new(ProductPlan::new(p, nextplan));
        }

        // Step 3: Add a selection plan for the predicate
        if !data.pred().is_empty() {
            p = Box::new(SelectPlan::new(p, data.pred().clone()));
        }

        // Step 4: Project on the field names
        p = Box::new(ProjectPlan::new(p, data.fields().to_vec()));
        p
    }
}

