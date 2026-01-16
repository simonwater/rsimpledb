use std::sync::{Arc, Mutex};

use super::LogIterator;
use crate::DbResult;
use crate::file::{BlockId, FileMgr, Page};

const INT_SIZE: usize = std::mem::size_of::<i32>();

#[derive(Clone)]
pub struct LogMgr {
    state: Arc<Mutex<LogMgrState>>,
}

impl LogMgr {
    pub fn new(fm: FileMgr, logfile: String) -> DbResult<Self> {
        let state = LogMgrState::new(fm, logfile)?;
        Ok(LogMgr {
            state: Arc::new(Mutex::new(state)),
        })
    }

    /// Ensures that the log record corresponding to `lsn` has been written
    /// to disk. All earlier log records will also be written.
    pub fn flush(&mut self, lsn: i32) -> DbResult<()> {
        let mut state = self.state.lock().unwrap();
        state.flush(lsn)
    }

    pub fn iterator(&mut self) -> DbResult<LogIterator> {
        let mut state = self.state.lock().unwrap();
        state.flush_now()?;
        let fm = state.fm.clone();
        let blk = state.currentblk.clone();
        LogIterator::new(fm, blk)
    }

    /// Appends a log record to the log buffer and returns its LSN.
    pub fn append(&mut self, logrec: &[u8]) -> DbResult<i32> {
        let mut state = self.state.lock().unwrap();
        state.append(logrec)
    }
}

struct LogMgrState {
    fm: FileMgr,
    logfile: String,
    logpage: Page,
    currentblk: BlockId,
    latest_lsn: i32,
    last_saved_lsn: i32,
}

impl LogMgrState {
    pub fn new(mut fm: FileMgr, logfile: String) -> DbResult<Self> {
        let mut lm = LogMgrState {
            fm: fm.clone(),
            logfile: logfile.clone(),
            logpage: Page::new(fm.block_size()),
            currentblk: BlockId::new(logfile.clone(), -1),
            latest_lsn: 0,
            last_saved_lsn: 0,
        };
        let logsize = fm.length(&logfile)?;
        lm.currentblk = if logsize == 0 {
            // creates the first block and returns it
            lm.append_new_block()?
        } else {
            let blk = BlockId::new(logfile.clone(), (logsize - 1) as i32);
            fm.read(&blk, &mut lm.logpage)?;
            blk
        };
        Ok(lm)
    }

    /// Ensures that the log record corresponding to `lsn` has been written
    /// to disk. All earlier log records will also be written.
    pub fn flush(&mut self, lsn: i32) -> DbResult<()> {
        if lsn >= self.last_saved_lsn {
            self.flush_now()?;
        }
        Ok(())
    }

    /// Appends a log record to the log buffer and returns its LSN.
    pub fn append(&mut self, logrec: &[u8]) -> DbResult<i32> {
        let mut boundary = self.logpage.get_int(0);
        let recsize = logrec.len() as i32;
        let bytesneeded = recsize + INT_SIZE as i32;
        if boundary - bytesneeded < INT_SIZE as i32 {
            self.flush_now()?;
            self.currentblk = self.append_new_block()?;
            boundary = self.logpage.get_int(0);
        }
        let recpos = boundary - bytesneeded;

        self.logpage.set_bytes(recpos as usize, logrec);
        self.logpage.set_int(0, recpos);
        self.latest_lsn += 1;
        Ok(self.latest_lsn)
    }

    fn append_new_block(&mut self) -> DbResult<BlockId> {
        let blk = self.fm.append(&self.logfile)?;
        self.logpage.set_int(0, self.fm.block_size() as i32);
        self.fm.write(&blk, &self.logpage)?;
        Ok(blk)
    }

    fn flush_now(&mut self) -> DbResult<()> {
        self.fm.write(&self.currentblk, &self.logpage)?;
        self.last_saved_lsn = self.latest_lsn;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::file::FileMgr;
    use crate::util::TempFileGuard;
    use std::path::PathBuf;

    #[test]
    fn log_mgr_test() {
        let db_dir = ".temp/lmdb";
        let _guard = TempFileGuard::new(db_dir);
        let fm = FileMgr::new(PathBuf::from(db_dir), 128).unwrap();
        let mut log_mgr = LogMgr::new(fm.clone(), "logfile".to_string()).unwrap();

        create_log_records(1, 35, &mut log_mgr);
        check_log_records(1, 35, &mut log_mgr);

        create_log_records(36, 70, &mut log_mgr);
        log_mgr.flush(65).unwrap();
        check_log_records(36, 70, &mut log_mgr);
    }

    fn check_log_records(start: i32, end: i32, log_mgr: &mut LogMgr) {
        let mut iter = log_mgr.iterator().unwrap();
        for i in (start..=end).rev() {
            let rec = iter.next().unwrap().unwrap();
            let p = Page::from_bytes(rec);
            let s = p.get_string(0);
            let ipos = 4 + s.len();
            let val = p.get_int(ipos);
            assert_eq!(s, format!("record{i}"));
            assert_eq!(val, i);
        }
    }

    fn create_log_records(start: i32, end: i32, log_mgr: &mut LogMgr) {
        for i in start..=end {
            let rec = create_log_record(format!("record{i}").as_str(), i);
            let lsn = log_mgr.append(&rec).unwrap();
            assert_eq!(lsn, i);
        }
        println!()
    }

    // Create a log record having two values: a string and an integer.
    fn create_log_record(s: &str, n: i32) -> Vec<u8> {
        let spos = 0;
        let ipos = spos + 4 + s.len(); // 4 bytes for length of string
        let rec = vec![0u8; ipos + 4]; // 4 bytes for integer
        let mut p = Page::from_bytes(rec);
        // write the string
        p.set_string(spos, s);
        p.set_int(ipos, n);
        p.contents().to_vec()
    }
}
