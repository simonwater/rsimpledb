use crate::DbResult;
use crate::parse::QueryData;
use crate::plan::Plan;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

/// The interface implemented by planners for the SQL select statement
pub trait QueryPlanner: Sync + Send {
    /// Creates a plan for the parsed query
    fn create_plan(
        &self,
        data: &QueryData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<Box<dyn Plan>>;
}
