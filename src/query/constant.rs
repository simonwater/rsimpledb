use std::cmp::Ordering;

/// The class that denotes values stored in the database
#[derive(Debug, Clone)]
pub enum Constant {
    Int(i32),
    String(String),
}

impl Constant {
    pub fn from_int(ival: i32) -> Self {
        Constant::Int(ival)
    }

    pub fn from_string(sval: String) -> Self {
        Constant::String(sval)
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Constant::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Constant::String(s) => Some(s),
            _ => None,
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => a == b,
            (Constant::String(a), Constant::String(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Constant {}

impl Ord for Constant {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => a.cmp(b),
            (Constant::String(a), Constant::String(b)) => a.cmp(b),
            _ => Ordering::Equal, // Different types are considered equal for simplicity
        }
    }
}

impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(i) => write!(f, "{}", i),
            Constant::String(s) => write!(f, "{}", s),
        }
    }
}
