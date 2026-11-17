use crate::{Expr, Int, Kind, Layout};

pub fn lower_int(expr: &Expr, layout: &Layout) -> Int {
    let pair = |a: &Expr, b: &Expr| {
        (
            Box::new(lower_int(a, layout)),
            Box::new(lower_int(b, layout)),
        )
    };
    match expr {
        Expr::Num(n) => Int::Lit(*n),
        Expr::Var(name) => Int::Reg(layout.index(Kind::Int, name)),
        Expr::Add(a, b) => {
            let (a, b) = pair(a, b);
            Int::Add(a, b)
        }
        Expr::Mul(a, b) => {
            let (a, b) = pair(a, b);
            Int::Mul(a, b)
        }
        Expr::Pow(a, b) => {
            let (a, b) = pair(a, b);
            Int::Pow(a, b)
        }
        _ => unreachable!("the checker typed {expr:?} as an int"),
    }
}
