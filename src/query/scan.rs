use crate::query::{Constant, UpdateScan};

/// The interface implemented by each query scan
pub trait Scan {
    /// Position the scan before its first record
    fn before_first(&mut self);

    /// Move the scan to the next record
    fn next(&mut self) -> bool;

    /// Return the value of the specified integer field in the current record
    fn get_int(&mut self, fldname: &str) -> i32;

    /// Return the value of the specified string field in the current record
    fn get_string(&mut self, fldname: &str) -> String;

    /// Return the value of the specified field in the current record
    fn get_val(&mut self, fldname: &str) -> Constant;

    /// Return true if the scan has the specified field
    fn has_field(&self, fldname: &str) -> bool;

    /// Close the scan and its subscans, if any
    fn close(&mut self);

    fn as_update_scan(&mut self) -> Option<&mut dyn UpdateScan> {
        None
    }
}
