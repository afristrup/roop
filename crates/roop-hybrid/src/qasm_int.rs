use crate::{Int, Layout};

pub fn qasm_int(int: &Int, layout: &Layout) -> String {
    let bin = |a, op, b| format!("({} {op} {})", qasm_int(a, layout), qasm_int(b, layout));
    match int {
        Int::Lit(n) => n.to_string(),
        Int::Reg(r) => format!("v_{}", layout.ints[*r]),
        Int::Add(a, b) => bin(a, "+", b),
        Int::Mul(a, b) => bin(a, "*", b),
        Int::Pow(a, b) => bin(a, "**", b),
        Int::Sub(a, b) => bin(a, "-", b),
        Int::Div(a, b) => bin(a, "/", b),
        Int::Rem(a, b) => bin(a, "%", b),
        Int::Xor(a, b) => bin(a, "^", b),
        Int::And(a, b) => bin(a, "&", b),
        Int::Or(a, b) => bin(a, "|", b),
    }
}
