use crate::record::SqlTypes;
use std::collections::HashMap;

#[derive(Clone)]
struct FieldInfo {
    type_: i32,
    length: i32,
}

/// The record schema of a table
#[derive(Clone)]
pub struct Schema {
    fields: Vec<String>,
    info: HashMap<String, FieldInfo>,
}

impl Schema {
    pub fn new() -> Self {
        Schema {
            fields: Vec::new(),
            info: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, fldname: &str, type_: i32, length: i32) {
        self.fields.push(fldname.to_string());
        self.info
            .insert(fldname.to_string(), FieldInfo { type_, length });
    }

    pub fn add_int_field(&mut self, fldname: &str) {
        self.add_field(fldname, SqlTypes::INTEGER, 0);
    }

    pub fn add_string_field(&mut self, fldname: &str, length: i32) {
        self.add_field(fldname, SqlTypes::VARCHAR, length);
    }

    pub fn add(&mut self, fldname: &str, sch: &Schema) {
        let type_ = sch.ftype(fldname);
        let length = sch.length(fldname);
        self.add_field(fldname, type_, length);
    }

    pub fn add_all(&mut self, sch: &Schema) {
        for fldname in sch.fields() {
            self.add(fldname, sch);
        }
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    pub fn has_field(&self, fldname: &str) -> bool {
        self.fields.iter().any(|f| f == fldname)
    }

    pub fn ftype(&self, fldname: &str) -> i32 {
        self.info.get(fldname).map(|f| f.type_).unwrap_or(0)
    }

    pub fn length(&self, fldname: &str) -> i32 {
        self.info.get(fldname).map(|f| f.length).unwrap_or(0)
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}
