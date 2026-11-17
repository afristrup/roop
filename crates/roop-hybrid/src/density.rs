use crate::Complex;

/// A partial density operator on `qubits` qubits, row-major. Qubit `q` is bit
/// `q` of a basis index.
#[derive(Debug, Clone, PartialEq)]
pub struct Density {
    pub qubits: usize,
    pub entries: Vec<Complex>,
}

impl Density {
    pub fn ground(qubits: usize) -> Self {
        let mut entries = vec![Complex::ZERO; 1 << (2 * qubits)];
        entries[0] = Complex::ONE;
        Self { qubits, entries }
    }

    pub fn dim(&self) -> usize {
        1 << self.qubits
    }

    pub fn trace(&self) -> f64 {
        (0..self.dim())
            .map(|i| self.entries[i * self.dim() + i].re)
            .sum()
    }

    pub fn add(&mut self, other: &Density) {
        for (a, b) in self.entries.iter_mut().zip(&other.entries) {
            *a = *a + *b;
        }
    }
}
