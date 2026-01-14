use crate::record::Schema;
use std::sync::Arc;

pub struct CreateTableData {
    tblname: String,
    sch: Arc<Schema>,
}

impl CreateTableData {
    pub fn new(tblname: String, sch: Arc<Schema>) -> Self {
        CreateTableData { tblname, sch }
    }

    pub fn table_name(&self) -> &str {
        &self.tblname
    }

    pub fn schema(&self) -> Arc<Schema> {
        self.sch.clone()
    }
}
