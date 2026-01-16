use crate::DbResult;
use crate::file::{BlockId, FileMgr, Page};

pub struct LogIterator {
    fm: FileMgr,
    blk: BlockId,
    p: Page,
    currentpos: usize,
    boundary: usize,
    has_error: bool,
}

impl LogIterator {
    pub fn new(fm: FileMgr, blk: BlockId) -> DbResult<Self> {
        let p = Page::new(fm.block_size());
        let mut it = LogIterator {
            fm,
            blk: blk.clone(),
            p,
            currentpos: 0,
            boundary: 0,
            has_error: false,
        };
        it.move_to_block(blk)?;
        Ok(it)
    }

    fn move_to_block(&mut self, blk: BlockId) -> DbResult<()> {
        self.fm.read(&blk, &mut self.p)?;
        let b = self.p.get_int(0) as usize;
        self.boundary = b;
        self.currentpos = b;
        self.blk = blk;
        Ok(())
    }

    pub fn has_next(&self) -> bool {
        !self.has_error && (self.currentpos < self.fm.block_size() || self.blk.number() > 0)
    }
}

impl Iterator for LogIterator {
    type Item = DbResult<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_next() {
            return None;
        }
        if self.currentpos == self.fm.block_size() {
            let prev_blk = BlockId::new(self.blk.file_name().to_string(), self.blk.number() - 1);
            match self.move_to_block(prev_blk) {
                Ok(_) => {}
                Err(e) => {
                    self.has_error = true;
                    return Some(Err(e));
                }
            }
        }
        let rec = self.p.get_bytes(self.currentpos);
        self.currentpos += std::mem::size_of::<i32>() + rec.len();
        Some(Ok(rec))
    }
}
