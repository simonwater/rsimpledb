use crate::DbResult;
use crate::file::BlockId;
use crate::index::IndexScan;
use crate::index::hash::hash_code;
use crate::query::Constant;
use crate::record::Layout;
use crate::record::RID;
use crate::record::RecordPage;
use crate::record::SqlTypes;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const MAX_DEPTH: i32 = 10;

pub struct ExtendableHashIndex {
    tx: Rc<RefCell<Transaction>>,
    layout: Arc<Layout>, // layout of the index records: (block, id, dataval)
    buckettbl: String,
    searchkey: Option<Constant>,
    dir_blk: BlockId, // dir[bucket] -> block
    cur_bucket_rp: Option<RecordPage>,
    cur_bucket_depth: i32,
    cur_bucket_slot: i32,
    current_rid: Option<RID>,
}

impl ExtendableHashIndex {
    /// Opens an extendable hash index for the specified index.
    pub fn new(tx: Rc<RefCell<Transaction>>, idxname: &str, layout: Arc<Layout>) -> DbResult<Self> {
        let dirtbl = format!("{}dir", idxname);
        let buckettbl = format!("{}bucket", idxname);

        let dir_blk = if tx.borrow_mut().size(&dirtbl)? == 0 {
            tx.borrow_mut().append(&buckettbl)?; // 桶第一个块
            tx.borrow_mut().append(&dirtbl)? // 目录块
        } else {
            BlockId::new(dirtbl, 0)
        };
        let index = ExtendableHashIndex {
            tx,
            layout,
            buckettbl,
            searchkey: None,
            dir_blk,
            cur_bucket_rp: None,
            cur_bucket_depth: 0,
            cur_bucket_slot: -1,
            current_rid: None,
        };

        Ok(index)
    }

    fn get_val(
        rp: &mut RecordPage,
        slot: i32,
        fldname: &str,
        layout: &Layout,
    ) -> DbResult<Constant> {
        if layout.schema().ftype(fldname) == SqlTypes::INTEGER {
            Ok(Constant::from_int(rp.get_int(slot, fldname)?))
        } else {
            Ok(Constant::from_string(rp.get_string(slot, fldname)?))
        }
    }
}

impl IndexScan for ExtendableHashIndex {
    /// Positions the index before the first record
    /// having the specified search key.
    fn before_first(&mut self, searchkey: &Constant) -> DbResult<()> {
        self.close();
        self.searchkey = Some(searchkey.clone());
        let bucket = hash_code(searchkey) % (1 << MAX_DEPTH);
        let mut tx = self.tx.borrow_mut();
        tx.pin(&self.dir_blk)?;
        let bucket_blknum = tx.get_int(&self.dir_blk, bucket as usize)?;
        let bucket_blk = BlockId::new(self.buckettbl.clone(), bucket_blknum);
        tx.pin(&bucket_blk)?;
        self.cur_bucket_depth = tx.get_int(&bucket_blk, 0)?;
        self.cur_bucket_rp = Some(RecordPage::new_with_start(
            Rc::clone(&self.tx),
            bucket_blk,
            Arc::clone(&self.layout),
            4,
        )?);
        self.cur_bucket_slot = -1;
        Ok(())
    }

    /// Moves the index to the next record having the
    /// search key specified in the before_first method.
    /// Returns false if there are no more such index records.
    fn next(&mut self) -> DbResult<bool> {
        let (Some(searchkey), Some(rp)) = (self.searchkey.as_ref(), self.cur_bucket_rp.as_mut())
        else {
            self.current_rid = None;
            return Ok(false);
        };

        self.cur_bucket_slot = rp.next_after(self.cur_bucket_slot)?;
        if self.cur_bucket_slot >= 0 {
            let dataval = Self::get_val(rp, self.cur_bucket_slot, "dataval", &*self.layout)?;
            if dataval == *searchkey {
                let blknum = rp.get_int(self.cur_bucket_slot, "block")?;
                let id = rp.get_int(self.cur_bucket_slot, "id")?;
                self.current_rid = Some(RID::new(blknum, id));
                return Ok(true);
            }
        }
        self.current_rid = None;
        Ok(false)
    }

    /// Returns the dataRID value stored in the current index record.
    fn get_data_rid(&mut self) -> DbResult<RID> {
        if let Some(ref rid) = self.current_rid {
            Ok(rid.clone())
        } else {
            Ok(RID::new(0, 0))
        }
    }

    /// Inserts an index record having the specified
    /// dataval and dataRID values.
    fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        unimplemented!()
    }

    /// Deletes the index record having the specified
    /// dataval and dataRID values.
    fn delete(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        unimplemented!()
    }
    /// Closes the index.
    fn close(&mut self) {
        // Nothing to do for now
        self.current_rid = None;
        self.cur_bucket_rp = None;
    }
}

impl Drop for ExtendableHashIndex {
    fn drop(&mut self) {
        self.close();
    }
}
