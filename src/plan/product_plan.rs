use crate::plan::Plan;
use crate::query::{Scan, ProductScan};
use crate::record::Schema;

/// The Plan class corresponding to the product relational algebra operator
pub struct ProductPlan {
    p1: Box<dyn Plan>,
    p2: Box<dyn Plan>,
    schema: Schema,
}

impl ProductPlan {
    /// Creates a new product node in the query tree
    pub fn new(p1: Box<dyn Plan>, p2: Box<dyn Plan>) -> Self {
        let mut schema = Schema::new();
        schema.add_all(&p1.schema());
        schema.add_all(&p2.schema());
        ProductPlan { p1, p2, schema }
    }
}

impl Plan for ProductPlan {
    /// Creates a product scan for this query
    fn open(&self) -> Box<dyn Scan> {
        let s1 = self.p1.open();
        let s2 = self.p2.open();
        Box::new(ProductScan::new(s1, s2))
    }

    /// Estimates the number of block accesses in the product
    /// Formula: B(product(p1,p2)) = B(p1) + R(p1)*B(p2)
    fn blocks_accessed(&self) -> i32 {
        self.p1.blocks_accessed() + (self.p1.records_output() * self.p2.blocks_accessed())
    }

    /// Estimates the number of output records in the product
    /// Formula: R(product(p1,p2)) = R(p1)*R(p2)
    fn records_output(&self) -> i32 {
        self.p1.records_output() * self.p2.records_output()
    }

    /// Estimates the distinct number of field values in the product
    fn distinct_values(&self, fldname: &str) -> i32 {
        if self.p1.schema().has_field(fldname) {
            self.p1.distinct_values(fldname)
        } else {
            self.p2.distinct_values(fldname)
        }
    }

    /// Returns the schema of the product
    fn schema(&self) -> Schema {
        self.schema.clone()
    }
}

