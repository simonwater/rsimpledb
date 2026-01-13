use crate::query::Constant;

pub struct InsertData {
    tblname: String,
    flds: Vec<String>,
    vals: Vec<Constant>,
}

impl InsertData {
    pub fn new(tblname: String, flds: Vec<String>, vals: Vec<Constant>) -> Self {
        InsertData { tblname, flds, vals }
    }

    pub fn table_name(&self) -> &str {
        &self.tblname
    }

    pub fn fields(&self) -> &[String] {
        &self.flds
    }

    pub fn vals(&self) -> &[Constant] {
        &self.vals
    }
}

