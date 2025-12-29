use std::collections::HashMap;
use std::sync::{Arc, Mutex, Condvar};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::file::BlockId;
use crate::tx::concurrency::LockAbortException;

const MAX_TIME_MS: u64 = 10000; // 10 seconds

/// The lock table, which provides methods to lock and unlock blocks
pub struct LockTable {
    locks: Arc<Mutex<HashMap<BlockId, i32>>>,
    condvar: Arc<Condvar>,
}

impl LockTable {
    pub fn new() -> Self {
        LockTable {
            locks: Arc::new(Mutex::new(HashMap::new())),
            condvar: Arc::new(Condvar::new()),
        }
    }

    /// Grant an SLock on the specified block
    pub fn s_lock(&self, blk: &BlockId) -> Result<(), LockAbortException> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        loop {
            let mut locks = self.locks.lock().unwrap();
            if !self.has_xlock(&locks, blk) {
                let val = self.get_lock_val(&locks, blk);
                locks.insert(blk.clone(), val + 1);
                return Ok(());
            }

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64 - start_time;

            if elapsed > MAX_TIME_MS {
                return Err(LockAbortException);
            }

            let _ = self.condvar.wait_timeout(
                locks,
                Duration::from_millis(MAX_TIME_MS - elapsed),
            );
        }
    }

    /// Grant an XLock on the specified block
    pub fn x_lock(&self, blk: &BlockId) -> Result<(), LockAbortException> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        loop {
            let mut locks = self.locks.lock().unwrap();
            if !self.has_other_slocks(&locks, blk) {
                locks.insert(blk.clone(), -1);
                return Ok(());
            }

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64 - start_time;

            if elapsed > MAX_TIME_MS {
                return Err(LockAbortException);
            }

            let _ = self.condvar.wait_timeout(
                locks,
                Duration::from_millis(MAX_TIME_MS - elapsed),
            );
        }
    }

    /// Release a lock on the specified block
    pub fn unlock(&self, blk: &BlockId) {
        let mut locks = self.locks.lock().unwrap();
        let val = self.get_lock_val(&locks, blk);
        if val > 1 {
            locks.insert(blk.clone(), val - 1);
        } else {
            locks.remove(blk);
            self.condvar.notify_all();
        }
    }

    fn has_xlock(&self, locks: &HashMap<BlockId, i32>, blk: &BlockId) -> bool {
        self.get_lock_val(locks, blk) < 0
    }

    fn has_other_slocks(&self, locks: &HashMap<BlockId, i32>, blk: &BlockId) -> bool {
        self.get_lock_val(locks, blk) > 1
    }

    fn get_lock_val(&self, locks: &HashMap<BlockId, i32>, blk: &BlockId) -> i32 {
        locks.get(blk).copied().unwrap_or(0)
    }
}

impl Default for LockTable {
    fn default() -> Self {
        Self::new()
    }
}

