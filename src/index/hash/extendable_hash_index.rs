use crate::DbResult;
use crate::file::BlockId;
use crate::index::IndexScan;
use crate::index::hash::BucketPage;
use crate::index::hash::DirPage;
use crate::index::hash::MAX_DEPTH;
use crate::index::hash::hash_code;
use crate::query::Constant;
use crate::record::Layout;
use crate::record::RID;
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct ExtendableHashIndex {
    tx: Rc<RefCell<Transaction>>,
    layout: Arc<Layout>, // layout of the index records: (block, id, dataval)
    buckettbl: String,
    searchkey: Option<Constant>,
    dir_page: DirPage,
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
        let dir_page = DirPage::new(Rc::clone(&tx), dir_blk.clone())?;
        let index = ExtendableHashIndex {
            tx,
            layout,
            buckettbl,
            searchkey: None,
            dir_page,
            bucket_page: None,
        };

        Ok(index)
    }
}

impl IndexScan for ExtendableHashIndex {
    fn before_first(&mut self, searchkey: &Constant) -> DbResult<()> {
        self.close();
        self.searchkey = Some(searchkey.clone());
        let bucketnum = hash_code(searchkey) % (1 << MAX_DEPTH);
        let bucket_blknum = self.dir_page.get_bucket_blknum(bucketnum)?;
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
        let Some(mut bucket_page) = self.bucket_page.as_mut() else {
            return Ok(());
        };

        if bucket_page.is_full()? {
            // Need to split the bucket
            bucket_page.split(&mut self.dir_page)?;
            // Reposition to the correct bucket
            self.before_first(dataval)?;
            bucket_page = self.bucket_page.as_mut().unwrap();
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataBase;
    use crate::record::{Layout, Schema};
    use crate::util::TempFileGuard;

    #[test]
    fn extendable_hash_test() {
        let db_dir = ".temp/extendable_hash_test";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new(db_dir).unwrap();
        let mut sch = Schema::new();
        sch.add_int_field("block");
        sch.add_int_field("id");
        sch.add_int_field("dataval");
        let layout = Layout::new(Arc::new(sch));
        let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));

        let mut index =
            ExtendableHashIndex::new(Rc::clone(&tx), "extendablehashidx", Arc::new(layout))
                .unwrap();

        // insert 10000 records
        for i in 0..10000 {
            let dataval = Constant::Int(i);
            let datarid = RID::new(i, i);
            index.insert(&dataval, &datarid).unwrap();
        }

        // search and verify
        for i in 100..1000 {
            let dataval = Constant::Int(i);
            let datarid = RID::new(i, i);
            index.before_first(&dataval).unwrap();
            let found = index.next().unwrap();
            assert!(found);
            let rid = index.get_data_rid().unwrap();
            assert_eq!(rid, datarid);
        }

        // delete some records
        for i in 10..20 {
            let n = i * 100;
            let dataval = Constant::Int(n);
            let datarid = RID::new(n, n);
            index.delete(&dataval, &datarid).unwrap();
        }

        // verify deletion
        for i in 10..20 {
            let n = i * 100;
            let dataval = Constant::Int(n);
            index.before_first(&dataval).unwrap();
            let found = index.next().unwrap();
            assert!(!found);
        }
    }
}
