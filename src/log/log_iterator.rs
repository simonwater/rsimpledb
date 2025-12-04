use crate::file::{BlockId, FileMgr, Page};

pub struct LogIterator<'a> {
    fm: &'a mut FileMgr,
    blk: BlockId,
    p: Page,
    currentpos: usize,
    boundary: usize,
}

impl<'a> LogIterator<'a> {
    pub fn new(fm: &'a mut FileMgr, blk: BlockId) -> Self {
        let p = Page::new(fm.block_size());
        let mut it = LogIterator {
            fm,
            blk: blk.clone(),
            p,
            currentpos: 0,
            boundary: 0,
        };
        it.move_to_block(blk);
        it
    }

    fn move_to_block(&mut self, blk: BlockId) {
        self.fm.read(&blk, &mut self.p);
        let b = self.p.get_int(0) as usize;
        self.boundary = b;
        self.currentpos = b;
        self.blk = blk;
    }

    pub fn has_next(&self) -> bool {
        self.currentpos < self.fm.block_size() || self.blk.number() > 0
    }
}

impl<'a> Iterator for LogIterator<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_next() {
            return None;
        }
        if self.currentpos == self.fm.block_size() {
            let prev_blk = BlockId::new(self.blk.file_name().to_string(), self.blk.number() - 1);
            self.move_to_block(prev_blk);
        }
        let rec = self.p.get_bytes(self.currentpos);
        self.currentpos += std::mem::size_of::<i32>() + rec.len();
        Some(rec)
    }
}
