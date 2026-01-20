use crate::DbResult;
use crate::file::BlockId;
use crate::query::Constant;
use crate::record::sql_types::INTEGER;
use crate::record::{Layout, RID};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

/// B-tree directory and leaf pages have many commonalities:
/// in particular, their records are stored in sorted order,
/// and pages split when full.
pub struct BTPage {
    tx: Rc<RefCell<Transaction>>,
    currentblk: Option<BlockId>,
    layout: Layout,
}

impl BTPage {
    /// Open a node for the specified B-tree block.
    pub fn new(
        tx: Rc<RefCell<Transaction>>,
        currentblk: BlockId,
        layout: Layout,
    ) -> DbResult<Self> {
        tx.borrow_mut().pin(&currentblk)?;

        Ok(BTPage {
            tx: tx.clone(),
            currentblk: Some(currentblk),
            layout,
        })
    }

    /// Calculate the position where the first record having
    /// the specified search key should be, then returns
    /// the position before it.
    pub fn find_slot_before(&self, searchkey: &Constant) -> DbResult<i32> {
        let mut slot = 0;
        while slot < self.get_num_recs()? {
            let data_val = self.get_data_val(slot)?;
            if data_val.partial_cmp(searchkey).unwrap() >= std::cmp::Ordering::Equal {
                break;
            }
            slot += 1;
        }
        Ok(slot - 1)
    }

    /// Close the page by unpinning its buffer.
    pub fn close(&mut self) {
        if let Some(blk) = self.currentblk.take() {
            self.tx.borrow_mut().unpin(&blk);
        }
    }

    /// Return true if the block is full.
    pub fn is_full(&self) -> DbResult<bool> {
        let num = self.get_num_recs()?;
        let pos = self.slotpos(num + 1);
        Ok(pos >= self.tx.borrow_mut().block_size() as i32)
    }

    /// Split the page at the specified position.
    pub fn split(&mut self, splitpos: i32, flag: i32) -> DbResult<BlockId> {
        let newblk = self.append_new(flag)?;
        let mut newpage = BTPage::new(self.tx.clone(), newblk.clone(), self.layout.clone())?;
        self.transfer_recs(splitpos, &mut newpage)?;
        newpage.set_flag(flag)?;
        newpage.close();
        Ok(newblk)
    }

    /// Return the dataval of the record at the specified slot.
    pub fn get_data_val(&self, slot: i32) -> DbResult<Constant> {
        self.get_val(slot, "dataval")
    }

    /// Return the value of the page's flag field
    pub fn get_flag(&self) -> DbResult<i32> {
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().get_int(blk, 0)
        } else {
            Ok(0)
        }
    }

    /// Set the page's flag field to the specified value
    pub fn set_flag(&mut self, val: i32) -> DbResult<()> {
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().set_int(blk, 0, val, true)?;
        }
        Ok(())
    }

    /// Append a new block to the end of the specified B-tree file
    pub fn append_new(&mut self, flag: i32) -> DbResult<BlockId> {
        let filename = if let Some(blk) = &self.currentblk {
            blk.file_name().to_string()
        } else {
            String::new()
        };

        let blk = self.tx.borrow_mut().append(&filename)?;
        self.tx.borrow_mut().pin(&blk)?;
        self.format(&blk, flag)?;
        Ok(blk)
    }

    /// Format a block
    pub fn format(&self, blk: &BlockId, flag: i32) -> DbResult<()> {
        // Set flag field
        self.tx.borrow_mut().set_int(blk, 0, flag, false)?;
        // Set number of records to 0
        self.tx.borrow_mut().set_int(blk, 4, 0, false)?;

        // Initialize all record slots with default values
        let recsize = self.layout.slot_size();
        let block_size = self.tx.borrow_mut().block_size();
        let mut pos = 8;

        while pos + recsize <= block_size as i32 {
            self.make_default_record(blk, pos as usize)?;
            pos += recsize;
        }
        Ok(())
    }

    fn make_default_record(&self, blk: &BlockId, pos: usize) -> DbResult<()> {
        for fldname in self.layout.schema().fields() {
            let offset = self.layout.offset(fldname) as usize;
            if self.layout.schema().ftype(fldname) == INTEGER {
                self.tx.borrow_mut().set_int(blk, pos + offset, 0, false)?;
            } else {
                self.tx
                    .borrow_mut()
                    .set_string(blk, pos + offset, "", false)?;
            }
        }
        Ok(())
    }

    // Methods called only by BTreeDir

    /// Return the block number stored in the index record at the specified slot
    pub fn get_child_num(&self, slot: i32) -> DbResult<i32> {
        self.get_int(slot, "block")
    }

    /// Insert a directory entry at the specified slot
    pub fn insert_dir(&mut self, slot: i32, val: &Constant, blknum: i32) -> DbResult<()> {
        self.insert(slot)?;
        self.set_val(slot, "dataval", val)?;
        self.set_int(slot, "block", blknum)?;
        Ok(())
    }

    // Methods called only by BTreeLeaf

    /// Return the dataRID value stored in the specified leaf index record
    pub fn get_data_rid(&self, slot: i32) -> DbResult<RID> {
        let blknum = self.get_int(slot, "block")?;
        let id = self.get_int(slot, "id")?;
        Ok(RID::new(blknum, id))
    }

    /// Insert a leaf index record at the specified slot
    pub fn insert_leaf(&mut self, slot: i32, val: &Constant, rid: &RID) -> DbResult<()> {
        self.insert(slot)?;
        self.set_val(slot, "dataval", val)?;
        self.set_int(slot, "block", rid.block_number())?;
        self.set_int(slot, "id", rid.slot())?;
        Ok(())
    }

    /// Delete the index record at the specified slot
    pub fn delete(&mut self, slot: i32) -> DbResult<()> {
        let num_recs = self.get_num_recs()?;
        for i in (slot + 1)..num_recs {
            self.copy_record(i, i - 1)?;
        }
        self.set_num_recs(num_recs - 1)?;
        Ok(())
    }

    /// Return the number of index records in this page
    pub fn get_num_recs(&self) -> DbResult<i32> {
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().get_int(blk, 4)
        } else {
            Ok(0)
        }
    }

    // Private methods

    fn get_int(&self, slot: i32, fldname: &str) -> DbResult<i32> {
        let pos = self.fldpos(slot, fldname);
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().get_int(blk, pos as usize)
        } else {
            Ok(0)
        }
    }

    fn get_string(&self, slot: i32, fldname: &str) -> DbResult<String> {
        let pos = self.fldpos(slot, fldname);
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().get_string(blk, pos as usize)
        } else {
            Ok(String::new())
        }
    }

    fn get_val(&self, slot: i32, fldname: &str) -> DbResult<Constant> {
        let field_type = self.layout.schema().ftype(fldname);
        if field_type == INTEGER {
            Ok(Constant::from_int(self.get_int(slot, fldname)?))
        } else {
            Ok(Constant::from_string(self.get_string(slot, fldname)?))
        }
    }

    fn set_int(&mut self, slot: i32, fldname: &str, val: i32) -> DbResult<()> {
        let pos = self.fldpos(slot, fldname);
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().set_int(blk, pos as usize, val, true)?;
        }
        Ok(())
    }

    fn set_string(&mut self, slot: i32, fldname: &str, val: &str) -> DbResult<()> {
        let pos = self.fldpos(slot, fldname);
        if let Some(blk) = &self.currentblk {
            self.tx
                .borrow_mut()
                .set_string(blk, pos as usize, val, true)?;
        }
        Ok(())
    }

    fn set_val(&mut self, slot: i32, fldname: &str, val: &Constant) -> DbResult<()> {
        let field_type = self.layout.schema().ftype(fldname);
        if field_type == INTEGER {
            if let Some(iv) = val.as_int() {
                self.set_int(slot, fldname, iv)?;
            }
        } else {
            if let Some(sv) = val.as_string() {
                self.set_string(slot, fldname, sv)?;
            }
        }
        Ok(())
    }

    fn set_num_recs(&mut self, n: i32) -> DbResult<()> {
        if let Some(blk) = &self.currentblk {
            self.tx.borrow_mut().set_int(blk, 4, n, true)?;
        }
        Ok(())
    }

    fn insert(&mut self, slot: i32) -> DbResult<()> {
        let num_recs = self.get_num_recs()?;
        for i in (slot..num_recs).rev() {
            self.copy_record(i, i + 1)?;
        }
        self.set_num_recs(num_recs + 1)?;
        Ok(())
    }

    fn copy_record(&mut self, from: i32, to: i32) -> DbResult<()> {
        for fldname in self.layout.schema().fields() {
            let val = self.get_val(from, fldname)?;
            self.set_val(to, fldname, &val)?;
        }
        Ok(())
    }

    fn transfer_recs(&mut self, slot: i32, dest: &mut BTPage) -> DbResult<()> {
        let mut destslot = 0;
        let source_slot = slot;

        while source_slot < self.get_num_recs()? {
            dest.insert(destslot)?;
            for fldname in self.layout.schema().fields() {
                let val = self.get_val(source_slot, fldname)?;
                dest.set_val(destslot, fldname, &val)?;
            }
            self.delete(source_slot)?;
            destslot += 1;
        }
        Ok(())
    }

    fn fldpos(&self, slot: i32, fldname: &str) -> i32 {
        let offset = self.layout.offset(fldname);
        self.slotpos(slot) + offset
    }

    fn slotpos(&self, slot: i32) -> i32 {
        let slotsize = self.layout.slot_size();
        8 + (slot * slotsize)
    }
}

impl Drop for BTPage {
    fn drop(&mut self) {
        self.close();
    }
}
