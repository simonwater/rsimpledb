use crate::DbResult;
use crate::index::IndexScan;
use crate::index::hash::hash_code;
use crate::query::Constant;
use crate::record::Layout;
use crate::record::RID;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const MAX_DEPTH: i32 = 10;

pub struct ExtendableHashIndex {
    tx: Rc<RefCell<Transaction>>,
    idxname: String,
    layout: Arc<Layout>, // layout of the index records: (block, id, dataval)
    idxdir: String,
    idxfile: String,
    searchkey: Option<Constant>,
}

impl ExtendableHashIndex {
    /// Opens an extendable hash index for the specified index.
    pub fn new(tx: Rc<RefCell<Transaction>>, idxname: &str, layout: Arc<Layout>) -> Self {
        ExtendableHashIndex {
            tx,
            idxname: idxname.to_string(),
            layout,
            idxdir: format!("{}_dir", idxname),
            idxfile: format!("{}_file", idxname),
            searchkey: None,
        }
    }
}

impl IndexScan for ExtendableHashIndex {
    /// Positions the index before the first record
    /// having the specified search key.
    fn before_first(&mut self, searchkey: &Constant) -> DbResult<()> {
        self.close();
        self.searchkey = Some(searchkey.clone());
        let bucket = hash_code(searchkey) % (1 << MAX_DEPTH);

        unimplemented!()
    }

    /// Moves the index to the next record having the
    /// search key specified in the before_first method.
    /// Returns false if there are no more such index records.
    fn next(&mut self) -> DbResult<bool> {
        unimplemented!()
    }

    /// Returns the dataRID value stored in the current index record.
    fn get_data_rid(&mut self) -> DbResult<RID> {
        unimplemented!()
    }

    /// Inserts an index record having the specified
    /// dataval and dataRID values.
    fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        unimplemented!()
    }

    /// Deletes the index record having the specified
    /// dataval and dataRID values.
    fn delete(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        unimplemented!()
    }
    /// Closes the index.
    fn close(&mut self) {
        // Nothing to do for now
    }
}

impl Drop for ExtendableHashIndex {
    fn drop(&mut self) {
        self.close();
    }
}
