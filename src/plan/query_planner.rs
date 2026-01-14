use std::sync::Arc;
use crate::parse::QueryData;
use crate::plan::Plan;
use crate::tx::Transaction;

/// The interface implemented by planners for the SQL select statement
pub trait QueryPlanner {
    /// Creates a plan for the parsed query
    fn create_plan(&self, data: &QueryData, tx: Arc<std::sync::Mutex<Transaction>>) -> Box<dyn Plan>;
}

