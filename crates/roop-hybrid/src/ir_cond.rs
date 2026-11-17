use crate::{Cond, Layout, ir_int};

pub fn ir_cond(cond: &Cond, layout: &Layout) -> String {
    let int = |i| ir_int(i, layout);
    let both = |a, op, b| format!("({} {op} {})", ir_cond(a, layout), ir_cond(b, layout));
    match cond {
        Cond::Lit(b) => b.to_string(),
        Cond::Bit(b) => layout.bits[*b].clone(),
        Cond::Le(a, b) => format!("({} <= {})", int(a), int(b)),
        Cond::Eq(a, b) => format!("({} = {})", int(a), int(b)),
        Cond::And(a, b) => both(a, "&&", b),
        Cond::Or(a, b) => both(a, "||", b),
        Cond::Xor(a, b) => both(a, "!=", b),
        Cond::Not(a) => format!("!{}", ir_cond(a, layout)),
    }
}
