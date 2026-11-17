use crate::{Int, Layout};

pub fn ir_int(int: &Int, layout: &Layout) -> String {
    let bin = |a, op, b| format!("({} {op} {})", ir_int(a, layout), ir_int(b, layout));
    match int {
        Int::Lit(n) => n.to_string(),
        Int::Reg(r) => layout.ints[*r].clone(),
        Int::Add(a, b) => bin(a, "+", b),
        Int::Mul(a, b) => bin(a, "*", b),
        Int::Pow(a, b) => bin(a, "^", b),
    }
}
