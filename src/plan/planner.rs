use crate::parse::{CreateCommand, Parser, QueryData, UpdateCommand};
use crate::plan::{Plan, QueryPlanner, UpdatePlanner};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

/// The object that executes SQL statements
pub struct Planner {
    qplanner: Box<dyn QueryPlanner>,
    uplanner: Box<dyn UpdatePlanner>,
}

impl Planner {
    pub fn new(qplanner: Box<dyn QueryPlanner>, uplanner: Box<dyn UpdatePlanner>) -> Self {
        Planner { qplanner, uplanner }
    }

    /// Creates a plan for an SQL select statement
    pub fn create_query_plan(&self, qry: &str, tx: Rc<RefCell<Transaction>>) -> Box<dyn Plan> {
        let mut parser = Parser::new(qry);
        let data = parser.query();
        self.verify_query(&data);
        self.qplanner.create_plan(&data, tx)
    }

    /// Executes an SQL insert, delete, modify, or create statement
    pub fn execute_update(&self, cmd: &str, tx: Rc<RefCell<Transaction>>) -> i32 {
        let mut parser = Parser::new(cmd);
        let data = parser.update_cmd();
        self.verify_update(&data);

        match data {
            UpdateCommand::Insert(insert_data) => self.uplanner.execute_insert(&insert_data, tx),
            UpdateCommand::Delete(delete_data) => self.uplanner.execute_delete(&delete_data, tx),
            UpdateCommand::Modify(modify_data) => self.uplanner.execute_modify(&modify_data, tx),
            UpdateCommand::Create(create_cmd) => match create_cmd {
                CreateCommand::Table(create_table_data) => {
                    self.uplanner.execute_create_table(&create_table_data, tx)
                }
                CreateCommand::View(create_view_data) => {
                    self.uplanner.execute_create_view(&create_view_data, tx)
                }
                CreateCommand::Index(create_index_data) => {
                    self.uplanner.execute_create_index(&create_index_data, tx)
                }
            },
        }
    }

    fn verify_query(&self, _data: &QueryData) {
        // SimpleDB does not verify queries, although it should
    }

    fn verify_update(&self, _data: &UpdateCommand) {
        // SimpleDB does not verify updates, although it should
    }
}
