/// An integer expression over int registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Int {
    Lit(i64),
    Reg(usize),
    Add(Box<Int>, Box<Int>),
    Mul(Box<Int>, Box<Int>),
    Pow(Box<Int>, Box<Int>),
}
