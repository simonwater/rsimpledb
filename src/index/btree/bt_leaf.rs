use super::bt_page::BTPage;
use super::dir_entry::DirEntry;
use crate::DbResult;
use crate::file::BlockId;
use crate::query::Constant;
use crate::record::{Layout, RID};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

/// An object that holds the contents of a B-tree leaf block.
pub struct BTreeLeaf {
    tx: Rc<RefCell<Transaction>>,
    layout: Layout,
    searchkey: Constant,
    contents: BTPage,
    currentslot: i32,
    filename: String,
}

impl BTreeLeaf {
    /// Opens a buffer to hold the specified leaf block.
    pub fn new(
        tx: Rc<RefCell<Transaction>>,
        blk: BlockId,
        layout: Layout,
        searchkey: Constant,
    ) -> DbResult<Self> {
        let filename = blk.file_name().to_string();
        let contents = BTPage::new(tx.clone(), blk, layout.clone())?;
        let currentslot = contents.find_slot_before(&searchkey)?;

        Ok(BTreeLeaf {
            tx,
            layout,
            searchkey,
            contents,
            currentslot,
            filename,
        })
    }

    /// Closes the leaf page.
    pub fn close(&mut self) {
        self.contents.close();
    }

    /// Moves to the next leaf record having the previously-specified search key.
    pub fn next(&mut self) -> DbResult<bool> {
        self.currentslot += 1;
        if self.currentslot >= self.contents.get_num_recs()? {
            self.try_overflow()
        } else if self
            .contents
            .get_data_val(self.currentslot)?
            .eq(&self.searchkey)
        {
            Ok(true)
        } else {
            self.try_overflow()
        }
    }

    /// Returns the dataRID value of the current leaf record.
    pub fn get_data_rid(&self) -> DbResult<RID> {
        self.contents.get_data_rid(self.currentslot)
    }

    /// Deletes the leaf record having the specified dataRID
    pub fn delete(&mut self, datarid: &RID) -> DbResult<()> {
        while self.next()? {
            if self.get_data_rid()?.eq(datarid) {
                self.contents.delete(self.currentslot)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Inserts a new leaf record having the specified dataRID
    pub fn insert(&mut self, datarid: &RID) -> DbResult<Option<DirEntry>> {
        if self.contents.get_flag()? >= 0
            && self
                .contents
                .get_data_val(0)?
                .partial_cmp(&self.searchkey)
                .unwrap()
                == std::cmp::Ordering::Greater
        {
            let firstval = self.contents.get_data_val(0)?;
            let newblk = self.contents.split(0, self.contents.get_flag()?)?;
            self.currentslot = 0;
            self.contents.set_flag(-1)?;
            self.contents
                .insert_leaf(self.currentslot, &self.searchkey, datarid)?;
            return Ok(Some(DirEntry::new(firstval, newblk.number())));
        }

        self.currentslot += 1;
        self.contents
            .insert_leaf(self.currentslot, &self.searchkey, datarid)?;

        if !self.contents.is_full()? {
            return Ok(None);
        }

        // Page is full, so split it
        let firstkey = self.contents.get_data_val(0)?;
        let lastkey = self
            .contents
            .get_data_val(self.contents.get_num_recs()? - 1)?;

        if lastkey.eq(&firstkey) {
            // Create an overflow block
            let newblk = self.contents.split(1, self.contents.get_flag()?)?;
            self.contents.set_flag(newblk.number())?;
            Ok(None)
        } else {
            let mut splitpos = self.contents.get_num_recs()? / 2;
            let mut splitkey = self.contents.get_data_val(splitpos)?;

            if splitkey.eq(&firstkey) {
                // Move right, looking for the next key
                while self.contents.get_data_val(splitpos)?.eq(&splitkey) {
                    splitpos += 1;
                }
                splitkey = self.contents.get_data_val(splitpos)?;
            } else {
                // Move left, looking for first entry having that key
                while self.contents.get_data_val(splitpos - 1)?.eq(&splitkey) {
                    splitpos -= 1;
                }
            }

            let newblk = self.contents.split(splitpos, -1)?;
            Ok(Some(DirEntry::new(splitkey, newblk.number())))
        }
    }

    fn try_overflow(&mut self) -> DbResult<bool> {
        let firstkey = self.contents.get_data_val(0)?;
        let flag = self.contents.get_flag()?;
        if !firstkey.eq(&self.searchkey) || flag < 0 {
            return Ok(false);
        }

        self.contents.close();
        let nextblk = BlockId::new(self.filename.clone(), flag);
        self.contents = BTPage::new(self.tx.clone(), nextblk, self.layout.clone())?;
        self.currentslot = 0;
        Ok(true)
    }
}

impl Drop for BTreeLeaf {
    fn drop(&mut self) {
        self.close();
    }
}
