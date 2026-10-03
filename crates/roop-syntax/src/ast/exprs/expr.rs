use crate::{BinOp, Place, UnOp};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    /// The empty stack, as the start value of an ancilla history.
    Empty,
    Variant(String, String),
    Place(Place),
    Unary(UnOp, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
}
