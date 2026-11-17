use crate::{Cond, Int};

/// A roop expression as an int or a bool of the hybrid IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(Int),
    Bit(Cond),
}
