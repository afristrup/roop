#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Named(String),
    Ref { mutable: bool, inner: Box<Type> },
    Array(Box<Type>, u64),
}
