use crate::query::Constant;

pub struct InsertData {
    tblname: String,
    flds: Vec<String>,
    rows: Vec<Vec<Constant>>,
}

impl InsertData {
    pub fn new(tblname: String, flds: Vec<String>, rows: Vec<Vec<Constant>>) -> Self {
        InsertData {
            tblname,
            flds,
            rows,
        }
    }

    pub fn table_name(&self) -> &str {
        &self.tblname
    }

    pub fn fields(&self) -> &[String] {
        &self.flds
    }

    pub fn rows(&self) -> &[Vec<Constant>] {
        &self.rows
    }
}
