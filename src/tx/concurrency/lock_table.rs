use crate::file::BlockId;
use crate::tx::concurrency::LockAbortException;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_TIME_MS: u64 = 10000; // 10 seconds

/// The lock table, which provides methods to lock and unlock blocks
#[derive(Clone)]
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
                .as_millis() as u64
                - start_time;

            if elapsed > MAX_TIME_MS {
                return Err(LockAbortException);
            }

            let _ = self
                .condvar
                .wait_timeout(locks, Duration::from_millis(MAX_TIME_MS - elapsed));
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
            // 通常有其他锁时便不能获得X锁，但有一种情况例外：如果只有自己持有S锁，则可以直接升级为X锁。所以简单判断是否为0不能区分s是自己还是别人持有
            // 现有逻辑x_lock调用前会先调用s_lock确保自己已经持有S锁，所以可以能区分locks中blk的值为1时表示自己持有s，大于1时表示其他事务也持有s
            if !self.has_other_slocks(&locks, blk) {
                locks.insert(blk.clone(), -1);
                return Ok(());
            }

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                - start_time;

            if elapsed > MAX_TIME_MS {
                return Err(LockAbortException);
            }

            let _ = self
                .condvar
                .wait_timeout(locks, Duration::from_millis(MAX_TIME_MS - elapsed));
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
