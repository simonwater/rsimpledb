use crate::query::{Scan, UpdateScan};
use crate::record::{Layout, Schema, TableScan};
use crate::tx::Transaction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// The max characters a tablename or fieldname can have
pub const MAX_NAME: i32 = 16;

/// The table manager
#[derive(Clone)]
pub struct TableMgr {
    tcat_layout: Arc<Layout>,
    fcat_layout: Arc<Layout>,
}

impl TableMgr {
    /// Create a new catalog manager for the database system
    pub fn new(is_new: bool, tx: Rc<RefCell<Transaction>>) -> Self {
        let mut tcat_schema = Schema::new();
        tcat_schema.add_string_field("tblname", MAX_NAME);
        tcat_schema.add_int_field("slotsize");
        let tcat_schema = Arc::new(tcat_schema);
        let tcat_layout = Layout::new(Arc::clone(&tcat_schema));

        let mut fcat_schema = Schema::new();
        fcat_schema.add_string_field("tblname", MAX_NAME);
        fcat_schema.add_string_field("fldname", MAX_NAME);
        fcat_schema.add_int_field("type");
        fcat_schema.add_int_field("length");
        fcat_schema.add_int_field("offset");
        let fcat_schema = Arc::new(fcat_schema);
        let fcat_layout = Layout::new(Arc::clone(&fcat_schema));

        let tm = TableMgr {
            tcat_layout: Arc::new(tcat_layout),
            fcat_layout: Arc::new(fcat_layout),
        };

        if is_new {
            tm.create_table("tblcat", tcat_schema, Rc::clone(&tx));
            tm.create_table("fldcat", fcat_schema, Rc::clone(&tx));
        }
        tm
    }

    /// Create a new table having the specified name and schema
    pub fn create_table(&self, tblname: &str, sch: Arc<Schema>, tx: Rc<RefCell<Transaction>>) {
        let layout = Layout::new(sch);
        // insert one record into tblcat
        let mut tcat = TableScan::new(Rc::clone(&tx), "tblcat", Arc::clone(&self.tcat_layout));
        tcat.insert();
        tcat.set_string("tblname", tblname);
        tcat.set_int("slotsize", layout.slot_size());
        tcat.close();

        // insert a record into fldcat for each field
        let mut fcat = TableScan::new(tx, "fldcat", self.fcat_layout.clone());
        for fldname in layout.schema().fields() {
            fcat.insert();
            fcat.set_string("tblname", tblname);
            fcat.set_string("fldname", fldname);
            fcat.set_int("type", layout.schema().ftype(fldname));
            fcat.set_int("length", layout.schema().length(fldname));
            fcat.set_int("offset", layout.offset(fldname));
        }
        fcat.close();
    }

    /// Retrieve the layout of the specified table from the catalog
    pub fn get_layout(&self, tblname: &str, tx: Rc<RefCell<Transaction>>) -> Layout {
        let mut size = -1;
        let mut tcat = TableScan::new(Rc::clone(&tx), "tblcat", self.tcat_layout.clone());
        while tcat.next() {
            if tcat.get_string("tblname") == tblname {
                size = tcat.get_int("slotsize");
                break;
            }
        }
        tcat.close();

        let mut sch = Schema::new();
        let mut offsets = HashMap::new();
        let mut fcat = TableScan::new(tx, "fldcat", self.fcat_layout.clone());
        while fcat.next() {
            if fcat.get_string("tblname") == tblname {
                let fldname = fcat.get_string("fldname");
                let fldtype = fcat.get_int("type");
                let fldlen = fcat.get_int("length");
                let offset = fcat.get_int("offset");
                offsets.insert(fldname.clone(), offset);
                sch.add_field(&fldname, fldtype, fldlen);
            }
        }
        fcat.close();
        Layout::from_metadata(Arc::new(sch), offsets, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataBase;
    use crate::util::TempFileGuard;

    #[test]
    fn table_mgr_test() {
        let db_dir = ".temp/tblmgrtest";
        let _guard = TempFileGuard::new(db_dir);
        let db: DataBase = DataBase::new(db_dir);
        let tx = Rc::new(RefCell::new(db.new_tx()));
        let tm = TableMgr::new(true, Rc::clone(&tx));

        let mut sch = Schema::new();
        sch.add_int_field("A");
        sch.add_string_field("B", 9);
        tm.create_table("MyTable", Arc::new(sch), Rc::clone(&tx));

        let layout = tm.get_layout("MyTable", Rc::clone(&tx));
        let size = layout.slot_size();
        assert_eq!(48, size);
        let sch2 = layout.schema();
        assert_eq!(true, sch2.has_field("A"));
        assert_eq!(4, sch2.ftype("A"));
        assert_eq!(true, sch2.has_field("B"));
        assert_eq!(12, sch2.ftype("B"));
        assert_eq!(9, sch2.length("B"));
    }
}
