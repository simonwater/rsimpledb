use crate::DbResult;
use crate::plan::Plan;
use crate::query::{Constant, Expression, Scan};
use crate::record::Schema;

/// A term is a comparison between two expressions
#[derive(Clone)]
pub struct Term {
    lhs: Expression,
    rhs: Expression,
}

impl Term {
    pub fn new(lhs: Expression, rhs: Expression) -> Self {
        Term { lhs, rhs }
    }

    /// Return true if both expressions evaluate to the same constant
    pub fn is_satisfied(&self, s: &mut dyn Scan) -> DbResult<bool> {
        let lhsval = self.lhs.evaluate(s)?;
        let rhsval = self.rhs.evaluate(s)?;
        Ok(lhsval == rhsval)
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    /// Calculate the extent to which selecting on the term reduces the number of records
    pub fn reduction_factor(&self, p: &dyn Plan) -> i32 {
        if self.lhs.is_field_name() && self.rhs.is_field_name() {
            let lhs_name = self.lhs.as_field_name().unwrap();
            let rhs_name = self.rhs.as_field_name().unwrap();
            std::cmp::max(p.distinct_values(lhs_name), p.distinct_values(rhs_name))
        } else if self.lhs.is_field_name() {
            let lhs_name = self.lhs.as_field_name().unwrap();
            p.distinct_values(lhs_name)
        } else if self.rhs.is_field_name() {
            let rhs_name = self.rhs.as_field_name().unwrap();
            p.distinct_values(rhs_name)
        } else {
            // otherwise, the term equates constants
            if let (Some(lhs_c), Some(rhs_c)) = (self.lhs.as_constant(), self.rhs.as_constant()) {
                if lhs_c == rhs_c { 1 } else { i32::MAX }
            } else {
                i32::MAX
            }
        }
    }

    /// Determine if this term is of the form "F=c" where F is the specified field
    pub fn equates_with_constant(&self, fldname: &str) -> Option<Constant> {
        if self.lhs.is_field_name()
            && self.lhs.as_field_name() == Some(fldname)
            && !self.rhs.is_field_name()
        {
            self.rhs.as_constant().cloned()
        } else if self.rhs.is_field_name()
            && self.rhs.as_field_name() == Some(fldname)
            && !self.lhs.is_field_name()
        {
            self.lhs.as_constant().cloned()
        } else {
            None
        }
    }

    /// Determine if this term is of the form "F1=F2" where F1 is the specified field
    pub fn equates_with_field(&self, fldname: &str) -> Option<String> {
        if self.lhs.is_field_name()
            && self.lhs.as_field_name() == Some(fldname)
            && self.rhs.is_field_name()
        {
            self.rhs.as_field_name().map(|s| s.to_string())
        } else if self.rhs.is_field_name()
            && self.rhs.as_field_name() == Some(fldname)
            && self.lhs.is_field_name()
        {
            self.lhs.as_field_name().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Return true if both expressions apply to the specified schema
    pub fn applies_to(&self, sch: &Schema) -> bool {
        self.lhs.applies_to(sch) && self.rhs.applies_to(sch)
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.lhs, self.rhs)
    }
}
