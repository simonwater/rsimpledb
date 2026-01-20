use crate::DbResult;
use crate::index::IndexScan;
use crate::query::{Constant, Scan, UpdateScan};
use crate::record::{Layout, RID, TableScan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub const NUM_BUCKETS: i32 = 100;

/// A static hash implementation of index.
/// A fixed number of buckets is allocated (currently 100),
/// and each bucket is implemented as a file of index records.
pub struct HashIndex {
    tx: Rc<RefCell<Transaction>>,
    idxname: String,
    layout: Arc<Layout>,
    searchkey: Option<Constant>,
    ts: Option<TableScan>,
    current_rid: Option<RID>,
}

impl HashIndex {
    /// Opens a hash index for the specified index.
    pub fn new(tx: Rc<RefCell<Transaction>>, idxname: &str, layout: Arc<Layout>) -> Self {
        HashIndex {
            tx,
            idxname: idxname.to_string(),
            layout,
            searchkey: None,
            ts: None,
            current_rid: None,
        }
    }

    /// Returns the cost of searching an index file.
    pub fn search_cost(numblocks: i32, _rpb: i32) -> i32 {
        numblocks / NUM_BUCKETS
    }
}

impl IndexScan for HashIndex {
    /// Positions the index before the first record
    /// having the specified search key.
    fn before_first(&mut self, searchkey: &Constant) -> DbResult<()> {
        self.close();
        self.searchkey = Some(searchkey.clone());

        let bucket = hash_code(searchkey) % NUM_BUCKETS;
        let tblname = format!("{}{}", self.idxname, bucket);

        let layout = Arc::new(self.layout.clone());
        let table_scan = TableScan::new(self.tx.clone(), &tblname, Arc::clone(&layout))?;
        self.ts = Some(table_scan);
        Ok(())
    }

    /// Moves the index to the next record having the
    /// search key specified in the before_first method.
    fn next(&mut self) -> DbResult<bool> {
        if let Some(searchkey) = &self.searchkey {
            if let Some(ref mut ts) = self.ts {
                while ts.next()? {
                    let dataval = ts.get_val("dataval")?;
                    if dataval == *searchkey {
                        // Cache the current RID for get_data_rid()
                        let blk_num = ts.get_int("block")?;
                        let id = ts.get_int("id")?;
                        self.current_rid = Some(RID::new(blk_num, id));
                        return Ok(true);
                    }
                }
            }
        }
        self.current_rid = None;
        Ok(false)
    }

    /// Returns the dataRID value stored in the current index record.
    fn get_data_rid(&mut self) -> DbResult<RID> {
        if let Some(ref mut ts) = self.ts {
            let blknum = ts.get_int("block")?;
            let id = ts.get_int("id")?;
            Ok(RID::new(blknum, id))
        } else {
            Ok(RID::new(0, 0))
        }
    }

    /// Inserts an index record having the specified
    /// dataval and dataRID values.
    fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        self.before_first(dataval)?;
        if let Some(ref mut ts) = self.ts {
            ts.insert()?;
            ts.set_int("block", datarid.block_number())?;
            ts.set_int("id", datarid.slot())?;
            ts.set_val("dataval", dataval)?;
        }
        Ok(())
    }

    /// Deletes the index record having the specified
    /// dataval and dataRID values.
    fn delete(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        self.before_first(dataval)?;
        while self.next()? {
            let current_rid = self.get_data_rid()?;
            if current_rid == *datarid {
                if let Some(ref mut ts) = self.ts {
                    ts.delete()?;
                }
                return Ok(());
            }
        }
        Ok(())
    }

    /// Closes the index.
    fn close(&mut self) {
        if let Some(mut ts) = self.ts.take() {
            ts.close();
        }
    }
}

/// Compute hash code for a Constant value
fn hash_code(searchkey: &Constant) -> i32 {
    match searchkey {
        Constant::Int(i) => *i,
        Constant::String(s) => {
            let mut hash: i32 = 0;
            for (i, c) in s.chars().enumerate() {
                hash = hash.wrapping_mul(31).wrapping_add(c as i32);
                if i > 10 {
                    break; // Limit string length for hash calculation
                }
            }
            hash
        }
    }
}

impl Drop for HashIndex {
    fn drop(&mut self) {
        self.close();
    }
}
