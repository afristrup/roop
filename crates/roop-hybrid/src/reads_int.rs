use crate::Int;

/// Whether `int` reads the int register `reg`.
pub fn reads_int(int: &Int, reg: Option<usize>) -> bool {
    match int {
        Int::Lit(_) => false,
        Int::Reg(r) => Some(*r) == reg,
        Int::Add(a, b) | Int::Mul(a, b) | Int::Pow(a, b) => reads_int(a, reg) || reads_int(b, reg),
    }
}
