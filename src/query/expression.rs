use crate::DbResult;
use crate::query::{Constant, Scan};
use crate::record::Schema;

/// An expression consisting of either a field name or a constant
#[derive(Clone)]
pub enum Expression {
    Field(String),
    Constant(Constant),
}

impl Expression {
    pub fn from_field(fldname: String) -> Self {
        Expression::Field(fldname)
    }

    pub fn from_constant(c: Constant) -> Self {
        Expression::Constant(c)
    }

    pub fn evaluate(&self, s: &mut dyn Scan) -> DbResult<Constant> {
        match self {
            Expression::Field(fldname) => s.get_val(fldname),
            Expression::Constant(c) => Ok(c.clone()),
        }
    }

    pub fn is_field_name(&self) -> bool {
        matches!(self, Expression::Field(_))
    }

    pub fn as_field_name(&self) -> Option<&str> {
        match self {
            Expression::Field(name) => Some(name),
            _ => None,
        }
    }

    pub fn as_constant(&self) -> Option<&Constant> {
        match self {
            Expression::Constant(c) => Some(c),
            _ => None,
        }
    }

    pub fn applies_to(&self, sch: &Schema) -> bool {
        match self {
            Expression::Constant(_) => true,
            Expression::Field(fldname) => sch.has_field(fldname),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(c) => write!(f, "{}", c),
            Expression::Field(name) => write!(f, "{}", name),
        }
    }
}
