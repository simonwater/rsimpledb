use crate::DbResult;
use crate::parse::{
    CreateIndexData, CreateTableData, CreateViewData, DeleteData, InsertData, ModifyData,
};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

/// The interface implemented by the planners for SQL insert, delete, and modify statements
pub trait UpdatePlanner: Sync + Send {
    /// Executes the specified insert statement, and returns the number of affected records
    fn execute_insert(&self, data: &InsertData, tx: Rc<RefCell<Transaction>>) -> DbResult<i32>;

    /// Executes the specified delete statement, and returns the number of affected records
    fn execute_delete(&self, data: &DeleteData, tx: Rc<RefCell<Transaction>>) -> DbResult<i32>;

    /// Executes the specified modify statement, and returns the number of affected records
    fn execute_modify(&self, data: &ModifyData, tx: Rc<RefCell<Transaction>>) -> DbResult<i32>;

    /// Executes the specified create table statement, and returns the number of affected records
    fn execute_create_table(
        &self,
        data: &CreateTableData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<i32>;

    /// Executes the specified create view statement, and returns the number of affected records
    fn execute_create_view(
        &self,
        data: &CreateViewData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<i32>;

    /// Executes the specified create index statement, and returns the number of affected records
    fn execute_create_index(
        &self,
        data: &CreateIndexData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<i32>;
}
