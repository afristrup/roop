use roop_syntax::BinOp;

/// Binds tighter as it grows; every level groups from the left.
pub fn precedence(op: BinOp) -> u8 {
    match op {
        BinOp::Or => 1,
        BinOp::And => 2,
        BinOp::BitOr => 3,
        BinOp::BitAnd => 4,
        BinOp::Eq | BinOp::Ne => 5,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 6,
        BinOp::Add | BinOp::Sub => 7,
        BinOp::Mul | BinOp::Div | BinOp::Rem => 8,
    }
}
