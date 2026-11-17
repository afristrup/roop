use crate::{Cond, Layout, qasm_int};

pub fn qasm_cond(cond: &Cond, layout: &Layout) -> String {
    let int = |i| qasm_int(i, layout);
    let both = |a, op, b| format!("({} {op} {})", qasm_cond(a, layout), qasm_cond(b, layout));
    match cond {
        Cond::Lit(b) => b.to_string(),
        Cond::Bit(b) => format!("(c[{b}] == 1)"),
        Cond::Le(a, b) => format!("({} <= {})", int(a), int(b)),
        Cond::Eq(a, b) => format!("({} == {})", int(a), int(b)),
        Cond::And(a, b) => both(a, "&&", b),
        Cond::Or(a, b) => both(a, "||", b),
        Cond::Not(a) => format!("!{}", qasm_cond(a, layout)),
    }
}
