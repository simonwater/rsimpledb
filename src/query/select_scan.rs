use crate::query::{Constant, Predicate, Scan, UpdateScan};
use crate::record::RID;

/// The scan class corresponding to the select relational algebra operator
pub struct SelectScan {
    s: Box<dyn Scan>,
    pred: Predicate,
}

impl SelectScan {
    /// Create a select scan having the specified underlying scan and predicate
    pub fn new(s: Box<dyn Scan>, pred: Predicate) -> Self {
        SelectScan { s, pred }
    }
}

impl Scan for SelectScan {
    fn before_first(&mut self) {
        self.s.before_first();
    }

    fn next(&mut self) -> bool {
        while self.s.next() {
            if self.pred.is_satisfied(self.s.as_mut()) {
                return true;
            }
        }
        false
    }

    fn get_int(&mut self, fldname: &str) -> i32 {
        self.s.get_int(fldname)
    }

    fn get_string(&mut self, fldname: &str) -> String {
        self.s.get_string(fldname)
    }

    fn get_val(&mut self, fldname: &str) -> Constant {
        self.s.get_val(fldname)
    }

    fn has_field(&self, fldname: &str) -> bool {
        self.s.has_field(fldname)
    }

    fn close(&mut self) {
        self.s.close();
    }

    fn as_update_scan(&mut self) -> Option<&mut dyn UpdateScan> {
        Some(self as &mut dyn UpdateScan)
    }
}

// Note: SelectScan can only implement UpdateScan if the underlying scan is UpdateScan
// This is a simplified version - in a full implementation, we'd need to check the type
impl UpdateScan for SelectScan {
    fn set_int(&mut self, fldname: &str, val: i32) {
        if let Some(us) = self.s.as_update_scan() {
            us.set_int(fldname, val);
        } else {
            panic!("underlying scan does not support updates");
        }
    }

    fn set_string(&mut self, fldname: &str, val: &str) {
        if let Some(us) = self.s.as_update_scan() {
            us.set_string(fldname, val);
        } else {
            panic!("underlying scan does not support updates");
        }
    }

    fn set_val(&mut self, fldname: &str, val: &Constant) {
        if let Some(us) = self.s.as_update_scan() {
            us.set_val(fldname, val);
        } else {
            panic!("underlying scan does not support updates");
        }
    }

    fn insert(&mut self) {
        if let Some(us) = self.s.as_update_scan() {
            us.insert();
        } else {
            panic!("underlying scan does not support updates");
        }
    }

    fn delete(&mut self) {
        if let Some(us) = self.s.as_update_scan() {
            us.delete();
        } else {
            panic!("underlying scan does not support updates");
        }
    }

    fn get_rid(&mut self) -> RID {
        if let Some(us) = self.s.as_update_scan() {
            us.get_rid()
        } else {
            RID::new(-1, -1)
        }
    }

    fn move_to_rid(&mut self, _rid: &RID) {
        if let Some(us) = self.s.as_update_scan() {
            us.move_to_rid(_rid);
        } else {
            panic!("underlying scan does not support updates");
        }
    }
}
