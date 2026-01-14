use crate::query::{Constant, Scan};

/// The scan class corresponding to the project relational algebra operator
pub struct ProjectScan {
    s: Box<dyn Scan>,
    fieldlist: Vec<String>,
}

impl ProjectScan {
    /// Create a project scan having the specified underlying scan and field list
    pub fn new(s: Box<dyn Scan>, fieldlist: Vec<String>) -> Self {
        ProjectScan { s, fieldlist }
    }
}

impl Scan for ProjectScan {
    fn before_first(&mut self) {
        self.s.before_first();
    }

    fn next(&mut self) -> bool {
        self.s.next()
    }

    fn get_int(&mut self, fldname: &str) -> i32 {
        self.s.get_int(fldname)
    }

    fn get_string(&mut self, fldname: &str) -> String {
        self.s.get_string(fldname)
    }

    fn get_val(&mut self, fldname: &str) -> Constant {
        self.s.get_val(fldname)
    }

    fn has_field(&self, fldname: &str) -> bool {
        self.fieldlist.contains(&fldname.to_string())
    }

    fn close(&mut self) {
        self.s.close();
    }
}
