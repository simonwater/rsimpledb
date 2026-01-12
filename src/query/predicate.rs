use crate::plan::Plan;
use crate::query::{Constant, Scan, Term};
use crate::record::Schema;

/// A predicate is a boolean combination of terms
#[derive(Clone)]
pub struct Predicate {
    terms: Vec<Term>,
}

impl Predicate {
    pub fn new() -> Self {
        Predicate { terms: Vec::new() }
    }

    pub fn from_term(term: Term) -> Self {
        Predicate { terms: vec![term] }
    }

    pub fn conjoin_with(&mut self, pred: Predicate) {
        self.terms.extend(pred.terms);
    }

    /// Return true if the predicate is satisfied by the specified scan
    pub fn is_satisfied(&self, s: &mut dyn Scan) -> bool {
        self.terms.iter().all(|term| term.is_satisfied(s))
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Calculate the extent to which selecting on the predicate reduces the number of records
    pub fn reduction_factor(&self, p: &dyn Plan) -> i32 {
        let mut factor = 1;
        for term in &self.terms {
            factor *= term.reduction_factor(p);
        }
        factor
    }

    /// Return the subpredicate that applies to the specified schema
    pub fn select_sub_pred(&self, sch: &Schema) -> Option<Predicate> {
        let mut result = Predicate::new();
        for term in &self.terms {
            if term.applies_to(sch) {
                result.terms.push(term.clone());
            }
        }
        if result.terms.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Return the subpredicate consisting of terms that apply to the union of the two schemas
    pub fn join_sub_pred(&self, sch1: &Schema, sch2: &Schema) -> Option<Predicate> {
        let mut newsch = sch1.clone();
        newsch.add_all(sch2);
        let mut result = Predicate::new();
        for term in &self.terms {
            if !term.applies_to(sch1) && !term.applies_to(sch2) && term.applies_to(&newsch) {
                result.terms.push(term.clone());
            }
        }
        if result.terms.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Determine if there is a term of the form "F=c" where F is the specified field
    pub fn equates_with_constant(&self, fldname: &str) -> Option<Constant> {
        for term in &self.terms {
            if let Some(c) = term.equates_with_constant(fldname) {
                return Some(c);
            }
        }
        None
    }

    /// Determine if there is a term of the form "F1=F2" where F1 is the specified field
    pub fn equates_with_field(&self, fldname: &str) -> Option<String> {
        for term in &self.terms {
            if let Some(fld) = term.equates_with_field(fldname) {
                return Some(fld);
            }
        }
        None
    }
}

impl Default for Predicate {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.terms.is_empty() {
            return Ok(());
        }
        let mut iter = self.terms.iter();
        write!(f, "{}", iter.next().unwrap())?;
        for term in iter {
            write!(f, " and {}", term)?;
        }
        Ok(())
    }
}
