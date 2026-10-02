use super::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum Place {
    Var(String),
    Field(Box<Place>, String),
    Index(Box<Place>, Box<Expr>),
}
