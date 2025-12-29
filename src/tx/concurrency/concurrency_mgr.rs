use crate::file::BlockId;
use crate::tx::concurrency::{LockAbortException, LockTable};
use std::collections::HashMap;

/// The concurrency manager for the transaction
pub struct ConcurrencyMgr {
    locktbl: LockTable,
    locks: HashMap<BlockId, String>,
}

impl ConcurrencyMgr {
    pub fn new(locktbl: LockTable) -> Self {
        ConcurrencyMgr {
            locktbl,
            locks: HashMap::new(),
        }
    }

    /// Obtain an SLock on the block, if necessary
    pub fn s_lock(&mut self, blk: &BlockId) -> Result<(), LockAbortException> {
        if !self.locks.contains_key(blk) {
            self.locktbl.s_lock(blk)?;
            self.locks.insert(blk.clone(), "S".to_string());
        }
        Ok(())
    }

    /// Obtain an XLock on the block, if necessary
    pub fn x_lock(&mut self, blk: &BlockId) -> Result<(), LockAbortException> {
        if !self.has_x_lock(blk) {
            self.s_lock(blk)?;
            self.locktbl.x_lock(blk)?;
            self.locks.insert(blk.clone(), "X".to_string());
        }
        Ok(())
    }

    /// Release all locks
    pub fn release(&mut self) {
        for blk in self.locks.keys() {
            self.locktbl.unlock(blk);
        }
        self.locks.clear();
    }

    fn has_x_lock(&self, blk: &BlockId) -> bool {
        self.locks
            .get(blk)
            .map(|locktype| locktype == "X")
            .unwrap_or(false)
    }
}
