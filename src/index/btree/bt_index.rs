use super::bt_dir::BTreeDir;
use super::bt_leaf::BTreeLeaf;
use super::bt_page::BTPage;
use crate::DbResult;
use crate::file::BlockId;
use crate::index::IndexScan;
use crate::query::Constant;
use crate::record::sql_types::INTEGER;
use crate::record::{Layout, RID, Schema};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// A B-tree implementation of the Index interface.
pub struct BTreeIndex {
    tx: Rc<RefCell<Transaction>>,
    dir_layout: Layout,
    leaf_layout: Layout,
    leaftbl: String,
    leaf: Option<BTreeLeaf>,
    rootblk: BlockId,
}

impl BTreeIndex {
    /// Opens a B-tree index for the specified index.
    pub fn new(tx: Rc<RefCell<Transaction>>, idxname: &str, leaf_layout: Layout) -> DbResult<Self> {
        let leaftbl = format!("{}leaf", idxname);
        let dirtbl = format!("{}dir", idxname);
        let rootblk = BlockId::new(dirtbl.clone(), 0);

        // Deal with the leaves
        if tx.borrow_mut().size(&leaftbl)? == 0 {
            let blk = tx.borrow_mut().append(&leaftbl)?;

            let mut node = BTPage::new(tx.clone(), blk.clone(), leaf_layout.clone())?;
            node.format(&blk, -1)?;
            node.close();
        }

        // Deal with the directory
        let mut dir_schema = Schema::new();
        dir_schema.add_int_field("block");
        let leaf_schema = leaf_layout.schema();
        dir_schema.add("dataval", &*leaf_schema);
        let dir_layout = Layout::new(Arc::new(dir_schema.clone()));

        if tx.borrow_mut().size(&dirtbl)? == 0 {
            // Create new root block
            tx.borrow_mut().append(&dirtbl)?;

            let mut node = BTPage::new(tx.clone(), rootblk.clone(), dir_layout.clone())?;
            node.format(&rootblk, 0)?;

            // Insert initial directory entry
            let fldtype = dir_schema.ftype("dataval");
            let minval = if fldtype == INTEGER {
                Constant::from_int(i32::MIN)
            } else {
                Constant::from_string(String::new())
            };

            node.insert_dir(0, &minval, 0)?;
            node.close();
        }

        Ok(BTreeIndex {
            tx,
            dir_layout,
            leaf_layout,
            leaftbl,
            leaf: None,
            rootblk,
        })
    }

    /// Estimate the number of block accesses required to find all index records
    /// having a particular search key.
    pub fn search_cost(numblocks: i32, rpb: i32) -> i32 {
        1 + (numblocks as f64).log(rpb as f64) as i32
    }
}

impl IndexScan for BTreeIndex {
    /// Traverse the directory to find the leaf block corresponding
    /// to the specified search key.
    fn before_first(&mut self, searchkey: &Constant) -> DbResult<()> {
        self.close();

        let mut root = BTreeDir::new(
            self.tx.clone(),
            self.rootblk.clone(),
            self.dir_layout.clone(),
        )?;
        let blknum = root.search(searchkey)?;
        root.close();

        let leafblk = BlockId::new(self.leaftbl.clone(), blknum);
        self.leaf = Some(BTreeLeaf::new(
            self.tx.clone(),
            leafblk,
            self.leaf_layout.clone(),
            searchkey.clone(),
        )?);
        Ok(())
    }

    /// Move to the next leaf record having the previously-specified search key.
    fn next(&mut self) -> DbResult<bool> {
        if let Some(ref mut leaf) = self.leaf {
            leaf.next()
        } else {
            Ok(false)
        }
    }

    /// Return the dataRID value from the current leaf record.
    fn get_data_rid(&mut self) -> DbResult<RID> {
        if let Some(ref leaf) = self.leaf {
            leaf.get_data_rid()
        } else {
            Ok(RID::new(0, 0))
        }
    }

    /// Insert the specified record into the index.
    fn insert(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        self.before_first(dataval)?;

        if let Some(mut leaf) = self.leaf.take() {
            if let Some(e) = leaf.insert(datarid)? {
                leaf.close();

                let mut root = BTreeDir::new(
                    self.tx.clone(),
                    self.rootblk.clone(),
                    self.dir_layout.clone(),
                )?;
                if let Some(e2) = root.insert(e)? {
                    root.make_new_root(e2)?;
                }
                root.close();
            } else {
                leaf.close();
            }
        }
        Ok(())
    }

    /// Delete the specified index record.
    fn delete(&mut self, dataval: &Constant, datarid: &RID) -> DbResult<()> {
        self.before_first(dataval)?;
        if let Some(mut leaf) = self.leaf.take() {
            leaf.delete(datarid)?;
            leaf.close();
        }
        Ok(())
    }

    /// Close the index by closing its open leaf page.
    fn close(&mut self) {
        if let Some(mut leaf) = self.leaf.take() {
            leaf.close();
        }
    }
}

impl Drop for BTreeIndex {
    fn drop(&mut self) {
        self.close();
    }
}
