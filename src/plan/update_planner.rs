use std::sync::Arc;
use crate::parse::{InsertData, DeleteData, ModifyData, CreateTableData, CreateViewData, CreateIndexData};
use crate::tx::Transaction;

/// The interface implemented by the planners for SQL insert, delete, and modify statements
pub trait UpdatePlanner {
    /// Executes the specified insert statement, and returns the number of affected records
    fn execute_insert(&self, data: &InsertData, tx: Arc<std::sync::Mutex<Transaction>>) -> i32;
    
    /// Executes the specified delete statement, and returns the number of affected records
    fn execute_delete(&self, data: &DeleteData, tx: Arc<std::sync::Mutex<Transaction>>) -> i32;
    
    /// Executes the specified modify statement, and returns the number of affected records
    fn execute_modify(&self, data: &ModifyData, tx: Arc<std::sync::Mutex<Transaction>>) -> i32;
    
    /// Executes the specified create table statement, and returns the number of affected records
    fn execute_create_table(&self, data: &CreateTableData, tx: Arc<std::sync::Mutex<Transaction>>) -> i32;
    
    /// Executes the specified create view statement, and returns the number of affected records
    fn execute_create_view(&self, data: &CreateViewData, tx: Arc<std::sync::Mutex<Transaction>>) -> i32;
    
    /// Executes the specified create index statement, and returns the number of affected records
    fn execute_create_index(&self, data: &CreateIndexData, tx: Arc<std::sync::Mutex<Transaction>>) -> i32;
}

