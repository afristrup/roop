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
    /// An array or stack whose length is a generic parameter, until the
    /// function is instantiated.
    Param {
        elem: Box<Type>,
        len: String,
        stack: bool,
    },
}
