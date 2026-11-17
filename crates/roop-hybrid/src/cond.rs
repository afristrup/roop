use crate::Int;

/// A boolean expression over bit and int registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cond {
    Lit(bool),
    Bit(usize),
    Le(Int, Int),
    Eq(Int, Int),
    And(Box<Cond>, Box<Cond>),
    Or(Box<Cond>, Box<Cond>),
    Not(Box<Cond>),
    Xor(Box<Cond>, Box<Cond>),
}
