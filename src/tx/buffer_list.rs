use crate::DbResult;
use crate::buffer::{Buffer, BufferMgr};
use crate::file::BlockId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Manage the transaction's currently-pinned buffers
pub struct BufferList {
    buffers: HashMap<BlockId, Arc<Mutex<Buffer>>>,
    pins: VecDeque<BlockId>,
    bm: BufferMgr,
}

impl BufferList {
    pub fn new(bm: BufferMgr) -> Self {
        BufferList {
            buffers: HashMap::new(),
            pins: VecDeque::new(),
            bm,
        }
    }

    /// Return the buffer pinned to the specified block
    pub fn get_buffer(&self, blk: &BlockId) -> Option<Arc<Mutex<Buffer>>> {
        self.buffers.get(blk).map(Arc::clone)
    }

    /// Pin the block and keep track of the buffer internally
    pub fn pin(&mut self, blk: &BlockId) -> DbResult<()> {
        let buff = self.bm.pin(blk)?;
        self.buffers.insert(blk.clone(), Arc::clone(&buff));
        self.pins.push_back(blk.clone());
        Ok(())
    }

    /// Unpin the specified block
    pub fn unpin(&mut self, blk: &BlockId) {
        if let Some(buff) = self.buffers.get(blk) {
            self.bm.unpin(Arc::clone(buff));
            self.pins.retain(|b| b != blk);
            if !self.pins.contains(blk) {
                self.buffers.remove(blk);
            }
        }
    }

    /// Unpin any buffers still pinned by this transaction
    pub fn unpin_all(&mut self) {
        for blk in &self.pins {
            if let Some(buff) = self.buffers.get(blk) {
                self.bm.unpin(Arc::clone(buff));
            }
        }
        self.buffers.clear();
        self.pins.clear();
    }
}
