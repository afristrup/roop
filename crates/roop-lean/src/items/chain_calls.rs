use roop_syntax::{Block, Expr, FnDef, Place, Stmt, StmtKind, UpdateOp};

/// The calls of a function that is a straight line of calls and plain updates,
/// in order, each with whether it is an `uncall`. `None` for anything with
/// control flow, array access or a guarded update, which can fail in places a
/// call-by-call proof does not look.
pub fn chain_calls(def: &FnDef) -> Option<Vec<(String, bool)>> {
    let mut calls = Vec::new();
    collect(&def.body, &mut calls)?;
    (!calls.is_empty()).then_some(calls)
}

fn collect(block: &Block, calls: &mut Vec<(String, bool)>) -> Option<()> {
    block
        .stmts
        .iter()
        .try_for_each(|stmt| statement(stmt, calls))
}

fn statement(stmt: &Stmt, calls: &mut Vec<(String, bool)>) -> Option<()> {
    match &stmt.kind {
        StmtKind::Call { callee, .. } => calls.push((callee.clone(), false)),
        StmtKind::Uncall { callee, .. } => calls.push((callee.clone(), true)),
        StmtKind::Ancilla { body, .. } => return collect(body, calls),
        StmtKind::Update {
            target: Place::Var(_),
            op: UpdateOp::Add | UpdateOp::Sub | UpdateOp::Xor,
            value,
        } if plain(value) => {}
        _ => return None,
    }
    Some(())
}

fn plain(expr: &Expr) -> bool {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::Place(Place::Var(_)) => true,
        Expr::Unary(_, e) => plain(e),
        Expr::Binary(a, _, b) => plain(a) && plain(b),
        _ => false,
    }
}
