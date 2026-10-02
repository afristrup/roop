use crate::{CheckError, Pending, flatten_ancillas, inverts, stmt_reads, stmt_writes};
use roop_syntax::{Block, Span, Stmt, StmtKind};
use std::collections::HashSet;

/// Straight-line matching: every update of the ancilla must be undone by its
/// exact inverse, in stack order, with no intervening write to anything the
/// update read. Anything the matcher cannot prove is rejected.
pub fn check_ancilla(name: &str, body: &Block, span: Span) -> Result<(), CheckError> {
    let mut stack: Vec<Pending> = Vec::new();
    for stmt in flatten_ancillas(body, name) {
        let mut writes = HashSet::new();
        stmt_writes(stmt, &mut writes);
        let touches = writes.contains(name);
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
    )
}
