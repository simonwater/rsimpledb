use crate::file::Page;
use crate::record::Schema;
use crate::record::sql_types::INTEGER;
use std::collections::HashMap;
use std::rc::Rc;

/// Description of the structure of a record
pub struct Layout {
    schema: Rc<Schema>,
    offsets: HashMap<String, i32>,
    slotsize: i32,
}

impl Layout {
    /// Create a Layout object from a schema
    pub fn new(schema: Rc<Schema>) -> Self {
        let mut offsets = HashMap::new();
        let mut pos = 4i32; // leave space for the empty/inuse flag (Integer.BYTES)
        for fldname in schema.fields() {
            offsets.insert(fldname.clone(), pos);
            pos += Self::length_in_bytes(&schema, &fldname);
        }
        Layout {
            schema,
            offsets,
            slotsize: pos,
        }
    }

    /// Create a Layout object from the specified metadata
    pub fn from_metadata(schema: Rc<Schema>, offsets: HashMap<String, i32>, slotsize: i32) -> Self {
        Layout {
            schema,
            offsets,
            slotsize,
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn offset(&self, fldname: &str) -> i32 {
        self.offsets.get(fldname).copied().unwrap_or(0)
    }

    pub fn slot_size(&self) -> i32 {
        self.slotsize
    }

    fn length_in_bytes(schema: &Schema, fldname: &str) -> i32 {
        let fldtype = schema.type_(fldname);
        if fldtype == INTEGER {
            4 // Integer.BYTES
        } else {
            // fldtype == VARCHAR
            Page::max_length_in_page(schema.length(fldname) as usize) as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_test() {
        let mut sch = Schema::new();
        sch.add_int_field("A");
        sch.add_string_field("B", 9);
        sch.add_int_field("C");
        let layout = Layout::new(Rc::new(sch));
        assert_eq!(4, layout.offset("A"));
        assert_eq!(8, layout.offset("B"));
        assert_eq!(48, layout.offset("C"));
        assert_eq!(12 + Page::max_length_in_page(9) as i32, layout.slot_size());
    }
}
