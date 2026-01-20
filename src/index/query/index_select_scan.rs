use crate::DbResult;
use crate::index::IndexScan;
use crate::query::{Constant, Scan, UpdateScan};
use crate::record::TableScan;

/// The scan class corresponding to the select relational algebra operator.
/// This scan uses an index to locate records matching a selection constant.
pub struct IndexSelectScan {
    ts: TableScan,
    idx: Box<dyn IndexScan>,
    val: Constant,
}

impl IndexSelectScan {
    /// Creates an index select scan for the specified index and selection constant.
    pub fn new(ts: TableScan, idx: Box<dyn IndexScan>, val: Constant) -> DbResult<Self> {
        let mut scan = IndexSelectScan { ts, idx, val };
        scan.before_first()?;
        Ok(scan)
    }
}

impl Scan for IndexSelectScan {
    /// Positions the scan before the first record.
    fn before_first(&mut self) -> DbResult<()> {
        self.idx.before_first(&self.val)?;
        Ok(())
    }

    /// Moves to the next record satisfying the selection constant.
    fn next(&mut self) -> DbResult<bool> {
        let ok = self.idx.next()?;
        if ok {
            let rid = self.idx.get_data_rid()?;
            self.ts.move_to_rid(&rid)?;
        }
        Ok(ok)
    }

    /// Returns the value of the field of the current data record.
    fn get_int(&mut self, fldname: &str) -> DbResult<i32> {
        self.ts.get_int(fldname)
    }

    /// Returns the string value of the field.
    fn get_string(&mut self, fldname: &str) -> DbResult<String> {
        self.ts.get_string(fldname)
    }

    /// Returns the constant value of the field.
    fn get_val(&mut self, fldname: &str) -> DbResult<Constant> {
        self.ts.get_val(fldname)
    }

    /// Returns whether the data record has the specified field.
    fn has_field(&self, fldname: &str) -> bool {
        self.ts.has_field(fldname)
    }

    /// Closes the scan by closing the index and the table scan.
    fn close(&mut self) {
        self.idx.close();
        self.ts.close();
    }
}
