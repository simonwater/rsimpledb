use crate::DbResult;
use crate::query::Constant;
use crate::record::RID;

/// An interface to traverse and modify an index.
/// This trait corresponds to the Java Index interface.
pub trait IndexScan {
    /// Positions the index before the first record
    /// having the specified search key.
    fn before_first(&mut self, searchkey: &Constant) -> DbResult<()>;

    /// Moves the index to the next record having the
    /// search key specified in the before_first method.
    /// Returns false if there are no more such index records.
    fn next(&mut self) -> DbResult<bool>;

    /// Returns the dataRID value stored in the current index record.
    fn get_data_rid(&mut self) -> DbResult<RID>;

    /// Inserts an index record having the specified
    /// dataval and dataRID values.
    fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()>;

    /// Deletes the index record having the specified
    /// dataval and dataRID values.
    fn delete(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()>;
    /// Closes the index.
    fn close(&mut self);
}
