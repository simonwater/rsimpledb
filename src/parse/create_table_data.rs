use crate::record::Schema;

pub struct CreateTableData {
    tblname: String,
    sch: Schema,
}

impl CreateTableData {
    pub fn new(tblname: String, sch: Schema) -> Self {
        CreateTableData { tblname, sch }
    }

    pub fn table_name(&self) -> &str {
        &self.tblname
    }

    pub fn schema(&self) -> &Schema {
        &self.sch
    }
}

