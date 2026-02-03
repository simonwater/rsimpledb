use crate::DbResult;
use crate::index::IndexScan;
use crate::index::btree::BTreeIndex;
use crate::index::hash::StaticHashIndex;
use crate::metadata::StatInfo;
use crate::record::{Layout, Schema, SqlTypes};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum IndexType {
    BTree,
    StaticHash,
    ExtendableHash,
}

impl std::fmt::Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::BTree => write!(f, "BTree"),
            IndexType::StaticHash => write!(f, "StaticHash"),
            IndexType::ExtendableHash => write!(f, "ExtendableHash"),
        }
    }
}

/// The information about an index
pub struct IndexInfo {
    idxname: String,
    fldname: String,
    tx: Rc<RefCell<Transaction>>,
    _tbl_schema: Arc<Schema>, // keep a reference to the target table schema
    idx_layout: Arc<Layout>,  // layout of the index records: (block, id, dataval)
    si: StatInfo,
    idx_type: IndexType,
}

impl IndexInfo {
    /// Create an IndexInfo object for the specified index
    pub fn new(
        idxname: String,
        fldname: String,
        tbl_schema: Arc<Schema>,
        tx: Rc<RefCell<Transaction>>,
        si: StatInfo,
        idx_type: IndexType,
    ) -> Self {
        let idx_layout = Arc::new(Self::create_idx_layout(&tbl_schema, &fldname));
        IndexInfo {
            idxname,
            fldname,
            tx,
            _tbl_schema: tbl_schema,
            idx_layout,
            si,
            idx_type,
        }
    }

    /// Open the index described by this object
    pub fn open(&self) -> DbResult<Box<dyn IndexScan>> {
        match self.idx_type {
            IndexType::BTree => Ok(Box::new(BTreeIndex::new(
                self.tx.clone(),
                &self.idxname,
                Arc::clone(&self.idx_layout),
            )?)),
            IndexType::StaticHash => Ok(Box::new(StaticHashIndex::new(
                self.tx.clone(),
                &self.idxname,
                self.idx_layout.clone(),
            ))),
            IndexType::ExtendableHash => {
                Ok(Box::new(crate::index::hash::ExtendableHashIndex::new(
                    self.tx.clone(),
                    &self.idxname,
                    self.idx_layout.clone(),
                )?))
            }
        }
    }

    /// Estimate the number of block accesses required
    pub fn blocks_accessed(&self) -> i32 {
        let rpb = self.tx.borrow_mut().block_size() as i32 / self.idx_layout.slot_size();
        let numblocks = self.si.records_output() / rpb;
        match self.idx_type {
            IndexType::BTree => BTreeIndex::search_cost(numblocks, rpb),
            IndexType::StaticHash => StaticHashIndex::search_cost(numblocks, rpb),
            IndexType::ExtendableHash => 2,
        }
    }

    /// Return the estimated number of records having a search key
    pub fn records_output(&self) -> i32 {
        self.si.records_output() / self.si.distinct_values(&self.fldname)
    }

    /// Return the distinct values for a specified field
    pub fn distinct_values(&self, fname: &str) -> i32 {
        if self.fldname == fname {
            1
        } else {
            self.si.distinct_values(fname)
        }
    }

    fn create_idx_layout(tbl_schema: &Schema, fldname: &str) -> Layout {
        let mut sch = Schema::new();
        sch.add_int_field("block");
        sch.add_int_field("id");
        if tbl_schema.ftype(fldname) == SqlTypes::INTEGER {
            sch.add_int_field("dataval");
        } else {
            let fldlen = tbl_schema.length(fldname);
            sch.add_string_field("dataval", fldlen);
        }
        Layout::new(Arc::new(sch))
    }
}
