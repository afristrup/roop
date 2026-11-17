use crate::{Cond, Expr, Kind, Layout, lower_int};

pub fn lower_cond(expr: &Expr, layout: &Layout) -> Cond {
    let int = |e| lower_int(e, layout);
    let cond = |e| Box::new(lower_cond(e, layout));
    match expr {
        Expr::Num(n) => Cond::Lit(*n == 1),
        Expr::Bool(b) => Cond::Lit(*b),
        Expr::Var(name) => Cond::Bit(layout.index(Kind::Bit, name)),
        Expr::Le(a, b) => Cond::Le(int(a), int(b)),
        Expr::Eq(a, b) => Cond::Eq(int(a), int(b)),
        Expr::And(a, b) => Cond::And(cond(a), cond(b)),
        Expr::Or(a, b) => Cond::Or(cond(a), cond(b)),
        Expr::Not(a) => Cond::Not(cond(a)),
        _ => unreachable!("the checker typed {expr:?} as a bit"),
    }
}
