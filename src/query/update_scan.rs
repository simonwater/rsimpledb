use crate::DbResult;
use crate::query::{Constant, Scan};
use crate::record::RID;

/// The interface implemented by all updateable scans
pub trait UpdateScan: Scan {
    /// Modify the field value of the current record
    fn set_val(&mut self, fldname: &str, val: &Constant) -> DbResult<()>;

    /// Modify the field value of the current record
    fn set_int(&mut self, fldname: &str, val: i32) -> DbResult<()>;

    /// Modify the field value of the current record
    fn set_string(&mut self, fldname: &str, val: &str) -> DbResult<()>;

    /// Insert a new record somewhere in the scan
    fn insert(&mut self) -> DbResult<()>;

    /// Delete the current record from the scan
    fn delete(&mut self) -> DbResult<()>;

    /// Return the id of the current record
    fn get_rid(&mut self) -> RID;

    /// Position the scan so that the current record has the specified id
    fn move_to_rid(&mut self, rid: &RID) -> DbResult<()>;
}
