use std::collections::HashSet;
use crate::query::Predicate;

/// Data for the SQL select statement
pub struct QueryData {
    fields: Vec<String>,
    tables: HashSet<String>,
    pred: Predicate,
}

impl QueryData {
    pub fn new(fields: Vec<String>, tables: HashSet<String>, pred: Predicate) -> Self {
        QueryData { fields, tables, pred }
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    pub fn tables(&self) -> &HashSet<String> {
        &self.tables
    }

    pub fn pred(&self) -> &Predicate {
        &self.pred
    }
}

impl std::fmt::Display for QueryData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "select ")?;
        for (i, fldname) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", fldname)?;
        }
        write!(f, " from ")?;
        let tables_vec: Vec<&String> = self.tables.iter().collect();
        for (i, tblname) in tables_vec.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", tblname)?;
        }
        let predstring = format!("{}", self.pred);
        if !predstring.is_empty() {
            write!(f, " where {}", predstring)?;
        }
        Ok(())
    }
}

