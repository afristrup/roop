#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Named(String),
    Ref {
        mutable: bool,
        inner: Box<Type>,
    },
    Array(Box<Type>, u64),
    /// A history stack of at most `u64` values.
    Stack(Box<Type>, u64),
}
