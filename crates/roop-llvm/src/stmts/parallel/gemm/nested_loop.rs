use crate::{const_int, unit_loop};
use roop_syntax::{Block, Expr, Place, StmtKind, UpdateOp};

/// A block that is `ancilla v = 0; from v == 0 { body } loop { v += 1; } until
/// v == n - 1; v -= n - 1;`: the variable, the length and the body.
pub fn nested_loop(block: &Block) -> Option<(&str, i64, &Block)> {
    let [only] = block.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Ancilla {
        name,
        init: Expr::Int(0),
        body: scope,
        ..
    } = &only.kind
    else {
        return None;
    };
    let [looped, restore] = scope.stmts.as_slice() else {
        return None;
    };
    let StmtKind::From {
        entry,
        body,
        step,
        until,
    } = &looped.kind
    else {
        return None;
    };
    let StmtKind::Update {
        target: Place::Var(restored),
        op: UpdateOp::Sub,
        value,
    } = &restore.kind
    else {
        return None;
    };
    let (var, length) = unit_loop(entry, step, until)?;
    (var == name && restored == name && const_int(value)? == length - 1)
        .then_some((var, length, body))
}
