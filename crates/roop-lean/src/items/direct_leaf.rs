use roop_syntax::{BinOp, Block, Expr, FnDef, Place, StmtKind, UpdateOp};

/// Whether a function can get a no-ancilla contract by direct simplification.
pub fn direct_leaf(def: &FnDef) -> bool {
    direct_block(&def.body)
}

fn direct_block(block: &Block) -> bool {
    block.stmts.iter().all(|stmt| match &stmt.kind {
        StmtKind::Update {
            target: Place::Var(_),
            op: UpdateOp::Add | UpdateOp::Sub | UpdateOp::Xor,
            value,
        } => no_division(value),
        StmtKind::Swap(Place::Var(_), Place::Var(_)) => true,
        StmtKind::If {
            cond,
            then_block,
            else_block,
            ..
        } => no_division(cond) && direct_block(then_block) && direct_block(else_block),
        _ => false,
    })
}

fn no_division(expr: &Expr) -> bool {
    match expr {
        Expr::Binary(_, BinOp::Div, _) => false,
        Expr::Unary(_, expr) | Expr::Cast(expr, _) => no_division(expr),
        Expr::Binary(lhs, _, rhs) => no_division(lhs) && no_division(rhs),
        _ => true,
    }
}
