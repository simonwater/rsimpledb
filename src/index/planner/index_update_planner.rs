use crate::metadata::MetadataMgr;
use crate::parse::{
    CreateIndexData, CreateTableData, CreateViewData, DeleteData, InsertData, ModifyData,
};
use crate::plan::UpdatePlanner;
use crate::plan::{Plan, SelectPlan, TablePlan};
use crate::tx::Transaction;
use crate::{DbError, DbResult};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// A modification of the basic update planner.
/// It dispatches each update statement to the corresponding index planner.
pub struct IndexUpdatePlanner {
    mdm: Arc<MetadataMgr>,
}

impl IndexUpdatePlanner {
    /// Creates a new index update planner.
    pub fn new(mdm: Arc<MetadataMgr>) -> Self {
        IndexUpdatePlanner { mdm }
    }
}

impl UpdatePlanner for IndexUpdatePlanner {
    /// Executes an insert statement, updating all affected indexes.
    fn execute_insert(&self, data: &InsertData, tx: Rc<RefCell<Transaction>>) -> DbResult<i32> {
        let tblname = data.table_name();
        let p = TablePlan::new(tx.clone(), &tblname, &self.mdm)?;

        // first, insert the record
        let mut s = p.open()?;
        if let Some(us) = s.as_update_scan() {
            // then modify each field, inserting an index record if appropriate
            let index_infos = self.mdm.get_index_info(&tblname, tx.clone())?;
            for row in data.rows() {
                us.insert()?;
                let rid = us.get_rid();
                let mut col_iter = row.into_iter();
                for fldname in data.fields() {
                    if let Some(val) = col_iter.next() {
                        us.set_val(&fldname, &val)?;
                        // Insert index record if there's an index on this field
                        if let Some(ii) = index_infos.get(fldname.as_str()) {
                            let mut idx = ii.open()?;
                            idx.before_first(&val)?;
                            idx.insert(&val, &rid)?;
                            idx.close();
                        }
                    }
                }
            }
        } else {
            return Err(DbError::Internal("plan does not produce an UpdateScan"));
        }
        s.close();
        Ok(1)
    }

    /// Executes a delete statement, updating all affected indexes.
    fn execute_delete(&self, data: &DeleteData, tx: Rc<RefCell<Transaction>>) -> DbResult<i32> {
        let tblname = data.table_name();
        let p = TablePlan::new(tx.clone(), &tblname, &self.mdm)?;
        let p = SelectPlan::new(Box::new(p), data.pred().clone());
        let indexes = self.mdm.get_index_info(&tblname, tx.clone())?;

        let mut s = p.open()?;
        let mut count = 0;
        if let Some(us) = s.as_update_scan() {
            while us.next()? {
                // first, delete the record's RID from every index
                let rid = us.get_rid();
                for (fldname, idx_info) in &indexes {
                    let val = us.get_val(fldname)?;
                    // Delete index record
                    let mut idx = idx_info.open()?;
                    idx.before_first(&val)?;
                    idx.delete(&val, &rid)?;
                    idx.close();
                }
                // then delete the record
                us.delete()?;
                count += 1;
            }
        } else {
            return Err(DbError::Internal("plan does not produce an UpdateScan"));
        }
        s.close();
        Ok(count)
    }

    /// Executes a modify statement, updating all affected indexes.
    fn execute_modify(&self, data: &ModifyData, tx: Rc<RefCell<Transaction>>) -> DbResult<i32> {
        let tblname = data.table_name();
        let fldname = data.target_field();
        let p = TablePlan::new(tx.clone(), &tblname, &self.mdm)?;
        let p = SelectPlan::new(Box::new(p), data.pred().clone());

        let indexes = self.mdm.get_index_info(&tblname, tx.clone())?;
        let idx_info = indexes.get(fldname);

        let mut s = p.open()?;
        let mut count = 0;
        if let Some(us) = s.as_update_scan() {
            while us.next()? {
                // first, update the record
                let newval = data.new_value().evaluate(us)?;
                let oldval = us.get_val(&fldname)?;
                us.set_val(&fldname, &newval)?;

                // then update the appropriate index, if it exists
                if let Some(ref ii) = idx_info {
                    let rid = us.get_rid();
                    let mut idx = ii.open()?;
                    idx.before_first(&oldval)?;
                    idx.delete(&oldval, &rid)?;
                    idx.before_first(&newval)?;
                    idx.insert(&newval, &rid)?;
                    idx.close();
                }
                count += 1;
            }
        } else {
            return Err(DbError::Internal("plan does not produce an UpdateScan"));
        }
        s.close();
        Ok(count)
    }

    /// Executes a create table statement.
    fn execute_create_table(
        &self,
        data: &CreateTableData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<i32> {
        self.mdm
            .create_table(&data.table_name(), data.schema(), tx)?;
        Ok(0)
    }

    /// Executes a create view statement.
    fn execute_create_view(
        &self,
        data: &CreateViewData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<i32> {
        self.mdm
            .create_view(&data.view_name(), &data.view_def(), tx)?;
        Ok(0)
    }

    /// Executes a create index statement.
    fn execute_create_index(
        &self,
        data: &CreateIndexData,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<i32> {
        self.mdm.create_index(
            &data.index_name(),
            &data.table_name(),
            &data.field_name(),
            tx,
        )?;
        Ok(0)
    }
}
