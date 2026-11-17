/// What `roop hybrid` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridEmit {
    Qasm,
    Ir,
    Report,
}
