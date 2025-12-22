use crate::file::{BlockId, FileMgr, Page};
use crate::log::LogMgr;

/// An individual buffer that wraps a page and stores metadata about its status:
/// - Associated disk block
/// - Pin count
/// - Modification status (txnum, lsn)
pub struct Buffer {
    fm: FileMgr,
    lm: LogMgr,
    contents: Page,
    blk: Option<BlockId>,
    pins: usize,
    txnum: i32, // -1 if not modified
    lsn: i32,   // -1 if not set
}

impl Buffer {
    pub fn new(fm: FileMgr, lm: LogMgr) -> Self {
        let blocksize = fm.block_size();
        Buffer {
            fm,
            lm,
            contents: Page::new(blocksize),
            blk: None,
            pins: 0,
            txnum: -1,
            lsn: -1,
        }
    }

    /// Returns a reference to the page contents
    pub fn contents(&self) -> &Page {
        &self.contents
    }

    /// Returns a mutable reference to the page contents
    pub fn contents_mut(&mut self) -> &mut Page {
        &mut self.contents
    }

    /// Returns a reference to the disk block allocated to this buffer
    pub fn block(&self) -> Option<&BlockId> {
        self.blk.as_ref()
    }

    /// Marks the buffer as modified by a transaction
    pub fn set_modified(&mut self, txnum: i32, lsn: i32) {
        self.txnum = txnum;
        if lsn >= 0 {
            self.lsn = lsn;
        }
    }

    /// Returns true if the buffer is currently pinned (pin count > 0)
    pub fn is_pinned(&self) -> bool {
        self.pins > 0
    }

    /// Returns the transaction id that modified this buffer (-1 if not modified)
    pub fn modifying_tx(&self) -> i32 {
        self.txnum
    }

    /// Reads the contents of the specified block into the buffer.
    /// If the buffer was dirty, its previous contents are written to disk first.
    pub fn assign_to_block(&mut self, b: BlockId) {
        self.flush();
        self.blk = Some(b.clone());
        self.fm.read(&b, &mut self.contents);
        self.pins = 0;
    }

    /// Writes the buffer to disk if it is dirty (txnum >= 0)
    pub fn flush(&mut self) {
        if self.txnum >= 0 {
            if let Some(ref blk) = self.blk {
                self.lm.flush(self.lsn);
                self.fm.write(blk, &self.contents);
                self.txnum = -1;
            }
        }
    }

    /// Increases the pin count
    pub fn pin(&mut self) {
        self.pins += 1;
    }

    /// Decreases the pin count
    pub fn unpin(&mut self) {
        if self.pins > 0 {
            self.pins -= 1;
        }
    }
}
