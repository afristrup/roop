#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Int,
    /// An unsigned byte: the compares and the division are unsigned.
    Byte,
    Float,
    Bool,
    Enum,
}
