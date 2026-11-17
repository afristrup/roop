use crate::{Expr, GateName, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Skip,
    Declare {
        kind: Kind,
        name: String,
        line: usize,
    },
    Gate {
        gate: GateName,
        qubits: Vec<String>,
        line: usize,
    },
    Assign {
        name: String,
        value: Expr,
        line: usize,
    },
    Measure {
        bit: String,
        qubit: String,
        line: usize,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        els: Vec<Stmt>,
        line: usize,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
}
