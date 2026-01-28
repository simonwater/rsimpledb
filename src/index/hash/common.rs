use crate::query::Constant;

/// Compute hash code for a Constant value
pub fn hash_code(searchkey: &Constant) -> i32 {
    match searchkey {
        Constant::Int(i) => *i,
        Constant::String(s) => {
            let mut hash: i32 = 0;
            for (i, c) in s.chars().enumerate() {
                hash = hash.wrapping_mul(31).wrapping_add(c as i32);
                if i > 10 {
                    break; // Limit string length for hash calculation
                }
            }
            hash
        }
    }
}
