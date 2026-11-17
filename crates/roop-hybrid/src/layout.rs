use crate::{Kind, Signature};

/// Where each name lives: a qubit index, a bit register or an int register.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layout {
    pub qubits: Vec<String>,
    pub bits: Vec<String>,
    pub ints: Vec<String>,
}

impl Layout {
    pub fn new(sig: &Signature) -> Self {
        let of = |kind| {
            sig.entries
                .iter()
                .filter(|e| e.1 == kind)
                .map(|e| e.0.clone())
                .collect()
        };
        Self {
            qubits: of(Kind::Qubit),
            bits: of(Kind::Bit),
            ints: of(Kind::Int),
        }
    }

    pub fn index(&self, kind: Kind, name: &str) -> usize {
        let names = match kind {
            Kind::Qubit => &self.qubits,
            Kind::Bit => &self.bits,
            Kind::Int => &self.ints,
        };
        names
            .iter()
            .position(|n| n == name)
            .expect("the checker declared every name")
    }
}
