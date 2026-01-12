use crate::query::{Scan, Constant};
use crate::record::RID;

/// The interface implemented by all updateable scans
pub trait UpdateScan: Scan {
    /// Modify the field value of the current record
    fn set_val(&mut self, fldname: &str, val: &Constant);
    
    /// Modify the field value of the current record
    fn set_int(&mut self, fldname: &str, val: i32);
    
    /// Modify the field value of the current record
    fn set_string(&mut self, fldname: &str, val: &str);
    
    /// Insert a new record somewhere in the scan
    fn insert(&mut self);
    
    /// Delete the current record from the scan
    fn delete(&mut self);
    
    /// Return the id of the current record
    fn get_rid(&self) -> RID;
    
    /// Position the scan so that the current record has the specified id
    fn move_to_rid(&mut self, rid: &RID);
}

