use crate::{
    CheckError, Mutability, Pending, counted_net, flatten_ancillas, inverts, settles, starts_zero,
    stmt_reads, stmt_writes,
};
use roop_syntax::{Block, Expr, Span, Stmt, StmtKind};
use std::collections::HashSet;

/// Straight-line matching: every update of the ancilla must be undone by its
/// exact inverse, in stack order, with no intervening write to anything the
/// update read. Anything the matcher cannot prove is rejected. An ancilla that
/// starts at zero may instead be given to the world with `keep`, which settles
/// everything before it, in control flow or not.
pub fn check_ancilla(
    name: &str,
    init: &Expr,
    body: &Block,
    span: Span,
    mutability: &Mutability,
) -> Result<(), CheckError> {
    let mut stmts = flatten_ancillas(body, name);
    if starts_zero(init)
        && let Some(last) = stmts.iter().rposition(|s| settles(s, name, mutability))
    {
        stmts.drain(..=last);
    }
    let nets: Vec<Option<Stmt>> = stmts.iter().map(|s| counted_net(s, name)).collect();
    let mut stack: Vec<Pending> = Vec::new();
    for (original, net) in stmts.iter().zip(&nets) {
        let mut writes = HashSet::new();
        stmt_writes(original, &mut writes, Some(mutability));
        let touches = writes.contains(name);
        let stmt = net.as_ref().unwrap_or(original);
        if touches && !is_straight_line(stmt) {
            return Err(CheckError::AncillaTouchedInControlFlow {
                name: name.into(),
                span: stmt.span,
            });
        }
        if touches
            && stack
                .last()
                .is_some_and(|top| !top.tainted && inverts(&top.stmt.kind, &stmt.kind))
        {
            stack.pop();
            continue;
        }
        writes.remove(name);
        for pending in &mut stack {
            pending.tainted |= !pending.reads.is_disjoint(&writes);
        }
        if touches {
            let mut reads = HashSet::new();
            stmt_reads(stmt, &mut reads);
            stack.push(Pending {
                stmt,
                reads,
                tainted: false,
            });
        }
    }
    if stack.is_empty() {
        Ok(())
    } else {
        Err(CheckError::AncillaNotRestored {
            name: name.into(),
            span,
        })
    }
}

fn is_straight_line(stmt: &Stmt) -> bool {
    matches!(
        stmt.kind,
        StmtKind::Update { .. }
            | StmtKind::Swap(..)
            | StmtKind::Call { .. }
            | StmtKind::Uncall { .. }
            | StmtKind::Send { .. }
            | StmtKind::Recv { .. }
            | StmtKind::Push { .. }
            | StmtKind::Pop { .. }
            | StmtKind::Keep(_)
    )
}
