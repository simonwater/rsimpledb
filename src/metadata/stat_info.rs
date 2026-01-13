/// A StatInfo object holds statistical information about a table
#[derive(Clone)]
pub struct StatInfo {
    num_blocks: i32,
    num_recs: i32,
}

impl StatInfo {
    /// Create a StatInfo object
    pub fn new(numblocks: i32, numrecs: i32) -> Self {
        StatInfo {
            num_blocks: numblocks,
            num_recs: numrecs,
        }
    }

    /// Return the estimated number of blocks in the table
    pub fn blocks_accessed(&self) -> i32 {
        self.num_blocks
    }

    /// Return the estimated number of records in the table
    pub fn records_output(&self) -> i32 {
        self.num_recs
    }

    /// Return the estimated number of distinct values for the specified field
    pub fn distinct_values(&self, _fldname: &str) -> i32 {
        1 + (self.num_recs / 3) // This is a complete guess
    }
}

