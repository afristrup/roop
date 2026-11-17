use crate::{Cond, Int};

/// What a roop name stands for in the hybrid program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    Qubit(usize),
    Qubits(Vec<usize>),
    Int(usize),
    Bit(usize),
    /// A value passed to a `&` parameter.
    IntValue(Int),
    BitValue(Cond),
}
