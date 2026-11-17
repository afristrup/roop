use crate::Int;

/// Whether `int` reads the int register `reg`.
pub fn reads_int(int: &Int, reg: Option<usize>) -> bool {
    match int {
        Int::Lit(_) => false,
        Int::Reg(r) => Some(*r) == reg,
        Int::Add(a, b)
        | Int::Sub(a, b)
        | Int::Mul(a, b)
        | Int::Div(a, b)
        | Int::Rem(a, b)
        | Int::Pow(a, b)
        | Int::Xor(a, b)
        | Int::And(a, b)
        | Int::Or(a, b) => reads_int(a, reg) || reads_int(b, reg),
    }
}
