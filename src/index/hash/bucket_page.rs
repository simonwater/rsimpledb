use crate::DbResult;
use crate::file::BlockId;
use crate::index::hash::DirPage;
use crate::index::hash::MAX_DEPTH;
use crate::index::hash::common::hash_code;
use crate::query::Constant;
use crate::record::Layout;
use crate::record::RID;
use crate::record::RecordPage;
use crate::record::SqlTypes;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct BucketPage {
    tx: Rc<RefCell<Transaction>>,
    layout: Arc<Layout>, // layout of the index records: (block, id, dataval)
    rp: RecordPage,
    searchkey: Constant,
    local_depth: i32,
    length: i32,
    cur_slot: i32,
    cur_rid: Option<RID>,
    capacity: i32,
}

impl BucketPage {
    pub fn new(
        tx: Rc<RefCell<Transaction>>,
        blk: BlockId,
        searchkey: Constant,
        layout: Arc<Layout>,
    ) -> DbResult<Self> {
        tx.borrow_mut().pin(&blk)?;
        let capacity = (tx.borrow_mut().block_size() as i32 - 8) / layout.slot_size();
        let local_depth = tx.borrow_mut().get_int(&blk, 0)?;
        let length = tx.borrow_mut().get_int(&blk, 4)?;
        let rp = RecordPage::new_with_start(Rc::clone(&tx), blk, Arc::clone(&layout), 8)?;

        Ok(BucketPage {
            tx: Rc::clone(&tx),
            layout,
            rp,
            searchkey,
            local_depth,
            length,
            cur_slot: -1,
            cur_rid: None,
            capacity,
        })
    }

    pub fn get_val(&mut self, fldname: &str) -> DbResult<Constant> {
        if self.layout.schema().ftype(fldname) == SqlTypes::INTEGER {
            Ok(Constant::from_int(self.rp.get_int(self.cur_slot, fldname)?))
        } else {
            Ok(Constant::from_string(
                self.rp.get_string(self.cur_slot, fldname)?,
            ))
        }
    }

    pub fn set_val(&mut self, fldname: &str, val: &Constant) -> DbResult<()> {
        if self.layout.schema().ftype(fldname) == SqlTypes::INTEGER {
            if let Some(i) = val.as_int() {
                self.rp.set_int(self.cur_slot, fldname, i)?;
            }
        } else {
            if let Some(s) = val.as_string() {
                self.rp.set_string(self.cur_slot, fldname, s)?;
            }
        }
        Ok(())
    }

    // Moves the index to the next record having the
    // search key specified in the before_first method.
    pub fn next(&mut self) -> DbResult<bool> {
        self.cur_slot = self.rp.next_after(self.cur_slot)?;
        while self.cur_slot >= 0 {
            let dataval = self.get_val("dataval")?;
            if dataval == self.searchkey {
                let blknum = self.rp.get_int(self.cur_slot, "block")?;
                let id = self.rp.get_int(self.cur_slot, "id")?;
                self.cur_rid = Some(RID::new(blknum, id));
                return Ok(true);
            }
            self.cur_slot = self.rp.next_after(self.cur_slot)?;
        }
        self.cur_rid = None;
        Ok(false)
    }

    pub fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        if !self.is_full()? {
            self.cur_slot = self.rp.insert_after(self.cur_slot)?;
            self.rp
                .set_int(self.cur_slot, "block", datarid.block_number())?;
            self.rp.set_int(self.cur_slot, "id", datarid.slot())?;
            self.set_val("dataval", dataval)?;
            self.set_length(self.length + 1)?;
        }
        Ok(())
    }

    /// Deletes the index record having the specified
    /// dataval and dataRID values.
    pub fn delete(&mut self) -> DbResult<()> {
        while self.next()? {
            let dataval = self.get_val("dataval")?;
            if dataval == self.searchkey {
                self.rp.delete(self.cur_slot)?;
                self.set_length(self.length - 1)?;
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn split(&mut self, dir_page: &mut DirPage) -> DbResult<()> {
        let new_local_depth = self.local_depth + 1;
        if new_local_depth > MAX_DEPTH {
            // Cannot split further, max depth reached
            unimplemented!("Maximum local depth reached; cannot split further");
        }
        let mut new_bucket_page = self.new_bucket_page()?;
        self.set_local_depth(new_local_depth)?;
        new_bucket_page.set_local_depth(new_local_depth)?;

        // Rehash records
        let mut original_length = self.length();
        self.cur_slot = self.rp.next_after(-1)?; // initialize for iteration
        while self.cur_slot >= 0 {
            let dataval = self.get_val("dataval")?;
            let bucketnum = hash_code(&dataval) % (1 << MAX_DEPTH);
            let bitmask = 1 << (new_local_depth - 1);
            if (bucketnum & bitmask) != 0 {
                // Move to new bucket
                let data_blknum = self.rp.get_int(self.cur_slot, "block")?;
                let data_slot = self.rp.get_int(self.cur_slot, "id")?;
                let cur_rid = RID::new(data_blknum, data_slot);
                new_bucket_page.insert(&dataval, &cur_rid)?;
                self.rp.delete(self.cur_slot)?;
                original_length -= 1;

                // update directory
                let new_bucket_blknum = new_bucket_page.rp.block().number();
                dir_page.set_bucket_blknum(bucketnum, new_bucket_blknum)?;
            }
            self.cur_slot = self.rp.next_after(self.cur_slot)?;
        }
        self.set_length(original_length)?;
        Ok(())
    }

    pub fn get_data_rid(&mut self) -> DbResult<RID> {
        if let Some(ref rid) = self.cur_rid {
            Ok(rid.clone())
        } else {
            Ok(RID::new(0, 0))
        }
    }

    pub fn is_full(&mut self) -> DbResult<bool> {
        Ok(self.length == self.capacity)
    }

    pub fn local_depth(&self) -> i32 {
        self.local_depth
    }

    pub fn set_local_depth(&mut self, depth: i32) -> DbResult<()> {
        self.local_depth = depth;
        self.tx
            .borrow_mut()
            .set_int(self.rp.block(), 0, depth, true)
    }

    pub fn length(&self) -> i32 {
        self.length
    }

    pub fn set_length(&mut self, length: i32) -> DbResult<()> {
        self.length = length;
        self.tx
            .borrow_mut()
            .set_int(self.rp.block(), 4, length, true)
    }

    fn new_bucket_page(&self) -> DbResult<BucketPage> {
        let cur_blk = self.rp.block();
        let buckettbl = cur_blk.file_name();
        let new_blk = self.tx.borrow_mut().append(buckettbl)?; // new bucket block
        let new_page = BucketPage::new(
            Rc::clone(&self.tx),
            new_blk,
            self.searchkey.clone(),
            Arc::clone(&self.layout),
        )?;
        Ok(new_page)
    }
}
