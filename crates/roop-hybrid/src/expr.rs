/// A classical expression. The checker decides whether it is an int or a bit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i64),
    Bool(bool),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}
