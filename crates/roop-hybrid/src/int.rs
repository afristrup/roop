/// An integer expression over int registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Int {
    Lit(i64),
    Reg(usize),
    Add(Box<Int>, Box<Int>),
    Sub(Box<Int>, Box<Int>),
    Mul(Box<Int>, Box<Int>),
    Div(Box<Int>, Box<Int>),
    Rem(Box<Int>, Box<Int>),
    Pow(Box<Int>, Box<Int>),
    Xor(Box<Int>, Box<Int>),
    And(Box<Int>, Box<Int>),
    Or(Box<Int>, Box<Int>),
}
