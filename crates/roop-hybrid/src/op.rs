use crate::{Cond, Gate, Int};

/// The hybrid IR. A `Kernel` is a run of gates the quantum device takes in one
/// go; everything else runs on the classical host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Kernel(Vec<Gate>),
    Measure {
        bit: usize,
        qubit: usize,
    },
    SetBit {
        bit: usize,
        value: Cond,
    },
    SetInt {
        reg: usize,
        value: Int,
    },
    If {
        cond: Cond,
        then: Vec<Op>,
        els: Vec<Op>,
    },
    While {
        cond: Cond,
        body: Vec<Op>,
    },
}
