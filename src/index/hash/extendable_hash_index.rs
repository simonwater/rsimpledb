use crate::DbResult;
use crate::file::BlockId;
use crate::index::IndexScan;
use crate::index::hash::bucket_page;
use crate::index::hash::bucket_page::BucketPage;
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
    bucket_page: Option<BucketPage>,
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
            bucket_page: None,
        };

        Ok(index)
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
        self.bucket_page = Some(BucketPage::new(
            Rc::clone(&self.tx),
            bucket_blk,
            searchkey.clone(),
            Arc::clone(&self.layout),
        )?);
        Ok(())
    }

    /// Moves the index to the next record having the
    /// search key specified in the before_first method.
    /// Returns false if there are no more such index records.
    fn next(&mut self) -> DbResult<bool> {
        if let Some(bucket_page) = self.bucket_page.as_mut() {
            bucket_page.next()
        } else {
            Ok(false)
        }
    }

    /// Returns the dataRID value stored in the current index record.
    fn get_data_rid(&mut self) -> DbResult<RID> {
        if let Some(bucket_page) = self.bucket_page.as_mut() {
            bucket_page.get_data_rid()
        } else {
            Ok(RID::new(0, 0))
        }
    }

    /// Inserts an index record having the specified
    /// dataval and dataRID values.
    fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        self.before_first(dataval)?;
        let Some(bucket_page) = self.bucket_page.as_mut() else {
            return Ok(());
        };

        bucket_page.insert(dataval, datarid)
    }

    /// Deletes the index record having the specified
    /// dataval and dataRID values.
    fn delete(&mut self, dataval: &Constant, _datarid: &RID) -> DbResult<()> {
        self.before_first(dataval)?;
        let Some(bucket_page) = self.bucket_page.as_mut() else {
            return Ok(());
        };
        bucket_page.delete()
    }
    /// Closes the index.
    fn close(&mut self) {
        // Nothing to do for now
        self.bucket_page = None
    }
}

impl Drop for ExtendableHashIndex {
    fn drop(&mut self) {
        self.close();
    }
}
