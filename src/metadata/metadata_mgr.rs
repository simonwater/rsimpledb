use crate::DbResult;
use crate::metadata::{IndexInfo, IndexMgr, IndexType, StatInfo, StatMgr, TableMgr, ViewMgr};
use crate::record::{Layout, Schema};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// The metadata manager
#[derive(Clone)]
pub struct MetadataMgr {
    tbl_mgr: TableMgr,
    view_mgr: ViewMgr,
    stat_mgr: StatMgr,
    idx_mgr: IndexMgr,
}

impl MetadataMgr {
    pub fn new(
        is_new: bool,
        tx: Rc<RefCell<Transaction>>,
        index_type: IndexType,
    ) -> DbResult<Self> {
        let tbl_mgr = TableMgr::new(is_new, Rc::clone(&tx))?;
        let view_mgr = ViewMgr::new(is_new, tbl_mgr.clone(), Rc::clone(&tx))?;
        let stat_mgr = StatMgr::new(tbl_mgr.clone(), Rc::clone(&tx))?;
        let idx_mgr = IndexMgr::new(is_new, tbl_mgr.clone(), stat_mgr.clone(), tx, index_type)?;

        Ok(MetadataMgr {
            tbl_mgr,
            view_mgr,
            stat_mgr,
            idx_mgr,
        })
    }

    pub fn create_table(
        &self,
        tblname: &str,
        sch: Arc<Schema>,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<()> {
        self.tbl_mgr.create_table(tblname, sch, tx)
    }

    pub fn get_layout(&self, tblname: &str, tx: Rc<RefCell<Transaction>>) -> DbResult<Layout> {
        self.tbl_mgr.get_layout(tblname, tx)
    }

    pub fn create_view(
        &self,
        viewname: &str,
        viewdef: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<()> {
        self.view_mgr.create_view(viewname, viewdef, tx)
    }

    pub fn get_view_def(
        &self,
        viewname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<Option<String>> {
        self.view_mgr.get_view_def(viewname, tx)
    }

    pub fn create_index(
        &self,
        idxname: &str,
        tblname: &str,
        fldname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<()> {
        self.idx_mgr.create_index(idxname, tblname, fldname, tx)?;
        Ok(())
    }

    pub fn get_index_info(
        &self,
        tblname: &str,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<HashMap<String, IndexInfo>> {
        self.idx_mgr.get_index_info(tblname, tx)
    }

    pub fn get_stat_info(
        &self,
        tblname: &str,
        layout: Arc<Layout>,
        tx: Rc<RefCell<Transaction>>,
    ) -> DbResult<StatInfo> {
        self.stat_mgr.get_stat_info(tblname, layout, tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataBase;
    use crate::query::UpdateScan;
    use crate::record::TableScan;
    use crate::util::TempFileGuard;
    use std::sync::Arc;

    #[test]
    fn mdm_test() {
        let db_dir = ".temp/mdmtest";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new_with_conf(db_dir, 1024, 1024, IndexType::BTree).unwrap();
        let tx = Rc::new(RefCell::new(db.new_tx().unwrap()));
        let mdm = db.md_mgr();

        // Part 1: Table Metadata
        let mut sch = Schema::new();
        sch.add_int_field("A");
        sch.add_string_field("B", 9);
        mdm.create_table("MyTable", Arc::new(sch), Rc::clone(&tx))
            .unwrap();
        let layout = mdm.get_layout("MyTable", Rc::clone(&tx)).unwrap();
        let layout = Arc::new(layout);
        let size = layout.slot_size();
        assert_eq!(48, size);
        let sch2 = layout.schema();
        assert_eq!(true, sch2.has_field("A"));
        assert_eq!(4, sch2.ftype("A"));
        assert_eq!(true, sch2.has_field("B"));
        assert_eq!(12, sch2.ftype("B"));
        assert_eq!(9, sch2.length("B"));

        // Part 2: Statistics Metadata
        let mut ts = TableScan::new(Rc::clone(&tx), "MyTable", Arc::clone(&layout)).unwrap();
        for i in 1..=50 {
            ts.insert().unwrap();
            ts.set_int("A", i).unwrap();
            ts.set_string("B", &format!("rec{i}")).unwrap();
        }
        let si = mdm
            .get_stat_info("MyTable", Arc::clone(&layout), Rc::clone(&tx))
            .unwrap();
        assert_eq!(3, si.blocks_accessed());
        assert_eq!(50, si.records_output());
        assert_eq!(17, si.distinct_values("A"));
        assert_eq!(17, si.distinct_values("B"));

        // Part 3: View Metadata
        let viewdef = "select B from MyTable where A = 1";
        mdm.create_view("view_a", viewdef, Rc::clone(&tx)).unwrap();
        assert_eq!(
            viewdef,
            mdm.get_view_def("view_a", Rc::clone(&tx)).unwrap().unwrap()
        );

        // Part 4: Index Metadata
        mdm.create_index("idx_a", "MyTable", "A", Rc::clone(&tx))
            .unwrap();
        mdm.create_index("idx_b", "MyTable", "B", Rc::clone(&tx))
            .unwrap();
        let idxmap = mdm.get_index_info("MyTable", Rc::clone(&tx)).unwrap();

        assert!(idxmap.contains_key("A"));
        assert!(idxmap.contains_key("B"));
    }
}
