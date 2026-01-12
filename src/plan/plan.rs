use crate::query::Scan;
use crate::record::Schema;

/// The interface implemented by each query plan
pub trait Plan {
    /// Opens a scan corresponding to this plan
    fn open(&self) -> Box<dyn Scan>;
    
    /// Returns an estimate of the number of block accesses
    fn blocks_accessed(&self) -> i32;
    
    /// Returns an estimate of the number of records in the query's output table
    fn records_output(&self) -> i32;
    
    /// Returns an estimate of the number of distinct values for the specified field
    fn distinct_values(&self, fldname: &str) -> i32;
    
    /// Returns the schema of the query
    fn schema(&self) -> Schema;
}

