use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::buffer::buffer::Buffer;
use crate::file::{BlockId, FileMgr};
use crate::log::LogMgr;

/// Exception thrown when buffer manager cannot pin a buffer within timeout
#[derive(Debug)]
pub struct BufferAbortException;

impl std::fmt::Display for BufferAbortException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferAbortException: No buffers available")
    }
}

impl std::error::Error for BufferAbortException {}

#[derive(Clone)]
pub struct BufferMgr {
    state: Arc<Mutex<BufferMgrState>>,
}

impl BufferMgr {
    pub fn new(fm: FileMgr, lm: LogMgr, numbuffs: usize) -> Self {
        BufferMgr {
            state: Arc::new(Mutex::new(BufferMgrState::new(fm, lm, numbuffs))),
        }
    }

    /// Returns the number of available (unpinned) buffers
    pub fn available(&self) -> usize {
        self.state.lock().unwrap().available()
    }

    /// Flushes the dirty buffers modified by the specified transaction
    pub fn flush_all(&self, txnum: i32) {
        self.state.lock().unwrap().flush_all(txnum);
    }

    /// Unpins the specified buffer. If pin count goes to zero, it becomes available.
    pub fn unpin(&self, buff_arc: Arc<Mutex<Buffer>>) {
        self.state.lock().unwrap().unpin(buff_arc);
    }

    /// Pins a buffer to the specified block, waiting if necessary.
    /// Returns BufferAbortException if no buffer becomes available within MAX_TIME.
    pub fn pin(&self, blk: &BlockId) -> Result<Arc<Mutex<Buffer>>, BufferAbortException> {
        self.state.lock().unwrap().pin(blk)
    }
}

/// Manages the pinning and unpinning of buffers to blocks.
struct BufferMgrState {
    bufferpool: Vec<Arc<Mutex<Buffer>>>,
    num_available: usize,
}

const MAX_TIME_MS: u128 = 10000; // 10 seconds

impl BufferMgrState {
    /// Creates a buffer manager with the specified number of buffer slots
    pub fn new(fm: FileMgr, lm: LogMgr, numbuffs: usize) -> Self {
        let mut bufferpool = Vec::with_capacity(numbuffs);
        for _ in 0..numbuffs {
            let buff = Buffer::new(fm.clone(), lm.clone());
            bufferpool.push(Arc::new(Mutex::new(buff)));
        }
        BufferMgrState {
            bufferpool,
            num_available: numbuffs,
        }
    }

    /// Returns the number of available (unpinned) buffers
    pub fn available(&self) -> usize {
        self.num_available
    }

    /// Flushes the dirty buffers modified by the specified transaction
    pub fn flush_all(&mut self, txnum: i32) {
        for buff_arc in &self.bufferpool {
            let mut buff = buff_arc.lock().unwrap();
            if buff.modifying_tx() == txnum {
                buff.flush();
            }
        }
    }

    /// Unpins the specified buffer. If pin count goes to zero, it becomes available.
    pub fn unpin(&mut self, buff_arc: Arc<Mutex<Buffer>>) {
        let mut buff = buff_arc.lock().unwrap();
        buff.unpin();
        if !buff.is_pinned() {
            self.num_available += 1;
        }
    }

    /// Pins a buffer to the specified block, waiting if necessary.
    /// Returns BufferAbortException if no buffer becomes available within MAX_TIME.
    pub fn pin(&mut self, blk: &BlockId) -> Result<Arc<Mutex<Buffer>>, BufferAbortException> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        loop {
            if let Some(buff) = self.try_to_pin(&blk) {
                return Ok(buff);
            }

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                - start_time;

            if elapsed > MAX_TIME_MS {
                return Err(BufferAbortException);
            }

            // Sleep a bit before trying again to avoid busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Tries to pin a buffer to the specified block.
    /// Returns None if no buffer is available.
    fn try_to_pin(&mut self, blk: &BlockId) -> Option<Arc<Mutex<Buffer>>> {
        if let Some(buff_arc) = self.find_existing_buffer(blk) {
            {
                let mut buff = buff_arc.lock().unwrap();
                if !buff.is_pinned() {
                    self.num_available -= 1;
                }
                buff.pin();
            }
            return Some(buff_arc);
        }

        if let Some(buff_arc) = self.choose_unpinned_buffer() {
            {
                let mut buff = buff_arc.lock().unwrap();
                buff.assign_to_block(blk.clone());
                self.num_available -= 1;
                buff.pin();
            }
            return Some(buff_arc);
        }

        None
    }

    /// Finds an existing buffer that's assigned to the given block
    fn find_existing_buffer(&self, blk: &BlockId) -> Option<Arc<Mutex<Buffer>>> {
        for buff_arc in &self.bufferpool {
            let buff = buff_arc.lock().unwrap();
            if let Some(b) = buff.block() {
                if b == blk {
                    return Some(buff_arc.clone());
                }
            }
        }
        None
    }

    /// Finds an unpinned buffer from the pool
    fn choose_unpinned_buffer(&self) -> Option<Arc<Mutex<Buffer>>> {
        for buff_arc in &self.bufferpool {
            let buff = buff_arc.lock().unwrap();
            if !buff.is_pinned() {
                return Some(buff_arc.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thread::MultiThreadRunner;
    use std::path::PathBuf;
    use std::thread;

    #[test]
    fn test_buffer_manager_concurrency() {
        // 1. 初始化文件管理器和缓冲区管理器
        let db_dir = PathBuf::from(".temp/bmdb");
        let blocksize = 400;
        let mut fm = FileMgr::new(db_dir.clone(), blocksize);
        let lm = LogMgr::new(fm.clone(), "testlog.log".to_string());
        let bm = BufferMgr::new(fm.clone(), lm.clone(), 10);

        // 2. 创建测试文件和块
        let filename = "testfile";
        if fm.length(filename) == 0 {
            for _ in 0..5 {
                fm.append(filename);
            }
        }

        // 3. 启动多个线程进行并发测试
        let headers = vec![format!("{:<12}", "Block Num")];
        let runner = MultiThreadRunner::new(10, headers);
        runner.excute(move |id| {
            let blk = BlockId::new(
                filename.to_string(),
                (id % 5) as i32, // 故意让线程竞争 blk
            );

            // --- 模拟数据库操作 ---
            // 1. Pin 一个页面
            let buff_arc = bm.pin(&blk).unwrap();
            // 2. 获取锁修改内容
            {
                let mut buf = buff_arc.lock().unwrap();
                let msg = format!("Data from thread {}", id);
                buf.contents_mut().set_string(0, &msg);
                buf.set_modified(id as i32, -1);
            } // 释放锁

            // 模拟一些处理耗时
            //thread::sleep(std::time::Duration::from_millis(10));

            // 3. Unpin
            bm.unpin(buff_arc);
            vec![format!("{:<12}", blk.number())]
        });
    }
}
