use super::bt_page::BTPage;
use super::dir_entry::DirEntry;
use crate::DbResult;
use crate::file::BlockId;
use crate::query::Constant;
use crate::record::Layout;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

/// A B-tree directory block.
pub struct BTreeDir {
    tx: Rc<RefCell<Transaction>>,
    layout: Layout,
    contents: BTPage,
    filename: String,
}

impl BTreeDir {
    /// Creates an object to hold the contents of the specified B-tree block.
    pub fn new(tx: Rc<RefCell<Transaction>>, blk: BlockId, layout: Layout) -> DbResult<Self> {
        let filename = blk.file_name().to_string();
        let contents = BTPage::new(tx.clone(), blk, layout.clone())?;

        Ok(BTreeDir {
            tx,
            layout,
            contents,
            filename,
        })
    }

    /// Closes the directory page.
    pub fn close(&mut self) {
        self.contents.close();
    }

    /// Returns the block number of the B-tree leaf block
    /// that contains the specified search key.
    pub fn search(&mut self, searchkey: &Constant) -> DbResult<i32> {
        let mut childblk = self.find_child_block(searchkey)?;
        while self.contents.get_flag()? > 0 {
            self.contents.close();
            self.contents = BTPage::new(self.tx.clone(), childblk.clone(), self.layout.clone())?;
            childblk = self.find_child_block(searchkey)?;
        }
        Ok(childblk.number())
    }

    /// Creates a new root block for the B-tree.
    pub fn make_new_root(&mut self, e: DirEntry) -> DbResult<()> {
        let firstval = self.contents.get_data_val(0)?;
        let level = self.contents.get_flag()?;
        let newblk = self.contents.split(0, level)?;

        let oldroot = DirEntry::new(firstval, newblk.number());
        self.insert_entry(&oldroot)?;
        self.insert_entry(&e)?;
        self.contents.set_flag(level + 1)
    }

    /// Inserts a new directory entry into the B-tree block.
    pub fn insert(&mut self, e: DirEntry) -> DbResult<Option<DirEntry>> {
        if self.contents.get_flag()? == 0 {
            self.insert_entry(&e)
        } else {
            let childblk = self.find_child_block(e.data_val())?;
            let mut child = BTreeDir::new(self.tx.clone(), childblk, self.layout.clone())?;
            let myentry = child.insert(e)?;
            child.close();

            if let Some(entry) = myentry {
                self.insert_entry(&entry)
            } else {
                Ok(None)
            }
        }
    }

    fn insert_entry(&mut self, e: &DirEntry) -> DbResult<Option<DirEntry>> {
        let newslot = 1 + self.contents.find_slot_before(e.data_val())?;
        self.contents
            .insert_dir(newslot, e.data_val(), e.block_number())?;

        if !self.contents.is_full()? {
            return Ok(None);
        }

        // Page is full, so split it
        let level = self.contents.get_flag()?;
        let num = self.contents.get_num_recs()?;
        let splitpos = num / 2;
        let splitval = self.contents.get_data_val(splitpos)?;
        let newblk = self.contents.split(splitpos, level)?;

        Ok(Some(DirEntry::new(splitval, newblk.number())))
    }

    fn find_child_block(&self, searchkey: &Constant) -> DbResult<BlockId> {
        let mut slot = self.contents.find_slot_before(searchkey)?;
        let dataval = self.contents.get_data_val(slot + 1)?;
        if dataval.eq(searchkey) {
            slot += 1;
        }

        let blknum = self.contents.get_child_num(slot)?;
        Ok(BlockId::new(self.filename.clone(), blknum))
    }
}

impl Drop for BTreeDir {
    fn drop(&mut self) {
        self.close();
    }
}
