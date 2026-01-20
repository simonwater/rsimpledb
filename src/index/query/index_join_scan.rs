use super::super::scan::IndexScan;
use crate::DbResult;
use crate::query::{Constant, Scan, UpdateScan};
use crate::record::TableScan;

/// The scan class corresponding to the index join relational algebra operator.
/// This scan performs a join using an index on the right-hand side.
pub struct IndexJoinScan {
    lhs: Box<dyn Scan>,
    idx: Box<dyn IndexScan>,
    joinfield: String,
    rhs: TableScan,
}

impl IndexJoinScan {
    /// Creates an index join scan for the specified LHS scan and RHS index.
    pub fn new(
        lhs: Box<dyn Scan>,
        idx: Box<dyn IndexScan>,
        joinfield: String,
        rhs: TableScan,
    ) -> DbResult<Self> {
        let mut scan = IndexJoinScan {
            lhs,
            idx,
            joinfield,
            rhs,
        };
        scan.before_first()?;
        Ok(scan)
    }
}

impl Scan for IndexJoinScan {
    /// Positions the scan before the first record.
    fn before_first(&mut self) -> DbResult<()> {
        self.lhs.before_first()?;
        self.lhs.next()?;
        self.reset_index()?;
        Ok(())
    }

    /// Moves the scan to the next record.
    fn next(&mut self) -> DbResult<bool> {
        loop {
            if self.idx.next()? {
                let rid = self.idx.get_data_rid()?;
                self.rhs.move_to_rid(&rid)?;
                return Ok(true);
            }
            if !self.lhs.next()? {
                return Ok(false);
            }
            self.reset_index()?;
        }
    }

    /// Returns the integer value of the specified field.
    fn get_int(&mut self, fldname: &str) -> DbResult<i32> {
        if self.rhs.has_field(fldname) {
            self.rhs.get_int(fldname)
        } else {
            self.lhs.get_int(fldname)
        }
    }

    /// Returns the constant value of the specified field.
    fn get_val(&mut self, fldname: &str) -> DbResult<Constant> {
        if self.rhs.has_field(fldname) {
            self.rhs.get_val(fldname)
        } else {
            self.lhs.get_val(fldname)
        }
    }

    /// Returns the string value of the specified field.
    fn get_string(&mut self, fldname: &str) -> DbResult<String> {
        if self.rhs.has_field(fldname) {
            self.rhs.get_string(fldname)
        } else {
            self.lhs.get_string(fldname)
        }
    }

    /// Returns true if the field is in either the LHS or RHS schema.
    fn has_field(&self, fldname: &str) -> bool {
        self.rhs.has_field(fldname) || self.lhs.has_field(fldname)
    }

    /// Closes the scan by closing its LHS scan, RHS index, and RHS table scan.
    fn close(&mut self) {
        self.lhs.close();
        self.idx.close();
        self.rhs.close();
    }
}

impl IndexJoinScan {
    fn reset_index(&mut self) -> DbResult<()> {
        let searchkey = self.lhs.get_val(&self.joinfield)?;
        self.idx.before_first(&searchkey)?;
        Ok(())
    }
}
