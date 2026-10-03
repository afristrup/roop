use crate::{BinOp, Place, Type, UnOp};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    /// A byte, `b'a'`, of type `u8`.
    Byte(u8),
    /// A string literal, the bytes of an array `[u8; N]` that only reads.
    Str(Vec<u8>),
    /// The empty stack, as the start value of an ancilla history.
    Empty,
    Variant(String, String),
    Place(Place),
    Unary(UnOp, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// `e as T`, between the numeric types.
    Cast(Box<Expr>, Type),
}
