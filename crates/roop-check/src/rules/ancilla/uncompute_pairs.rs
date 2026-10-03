use crate::{
    Mutability, flatten_ancillas, inverts, is_straight_line, starts_zero, stmt_reads, stmt_writes,
};
use roop_syntax::{Block, Expr, Place, Stmt, StmtKind};
use std::collections::HashSet;

struct Open<'a> {
    stmt: &'a Stmt,
    reads: HashSet<&'a str>,
    tainted: bool,
    /// Whether the ancilla was still at its start, zero, when it was made.
    first: bool,
}

/// Whether a statement is `call f(.., name, ..)` or `uncall` of it, with the
/// ancilla the only thing `f` may write, so that undoing the call is no more than
/// zeroing the ancilla.
fn computes(stmt: &Stmt, name: &str, mutability: &Mutability, effectful: &HashSet<&str>) -> bool {
    let (StmtKind::Call { callee, args, .. } | StmtKind::Uncall { callee, args, .. }) = &stmt.kind
    else {
        return false;
    };
    let Some(written) = mutability.get(callee.as_str()) else {
        return false;
    };
    let is_name = |arg: &Expr| matches!(arg, Expr::Place(Place::Var(v)) if v == name);
    let writes_name = args.iter().zip(written).any(|(a, w)| *w && is_name(a));
    let writes_only_name = args.iter().zip(written).all(|(a, w)| !*w || is_name(a));
    !effectful.contains(callee.as_str()) && writes_name && writes_only_name
}

/// The pairs of statements in the scope of the ancilla `name` that make its
/// value and take it off again: `call f(name, ..)` and `uncall f(name, ..)`, in
/// either order, with nothing in between that writes what `f` reads, and with the
/// ancilla at zero before the first. Between them it holds `f` of those
/// arguments, so a run that goes through both can leave it at zero instead of
/// computing `f` backward; and so can one that goes through them in the other
/// order. Empty when the ancilla does not start at zero, or when any statement
/// that writes it is not a straight-line one. `effectful` are the functions that
/// do more than write their arguments.
pub fn uncompute_pairs<'a>(
    name: &str,
    init: &Expr,
    body: &'a Block,
    mutability: &Mutability,
    effectful: &HashSet<&str>,
) -> Vec<(&'a Stmt, &'a Stmt)> {
    if !starts_zero(init) {
        return Vec::new();
    }
    let mut open: Vec<Open<'a>> = Vec::new();
    let mut pairs = Vec::new();
    for stmt in flatten_ancillas(body, name) {
        let mut writes = HashSet::new();
        stmt_writes(stmt, &mut writes, Some(mutability));
        let touches = writes.contains(name);
        if touches && !is_straight_line(stmt) {
            return Vec::new();
        }
        if touches
            && open
                .last()
                .is_some_and(|top| !top.tainted && inverts(&top.stmt.kind, &stmt.kind))
        {
            let top = open.pop().expect("a statement to close");
            if top.first
                && computes(top.stmt, name, mutability, effectful)
                && computes(stmt, name, mutability, effectful)
            {
                pairs.push((top.stmt, stmt));
            }
            continue;
        }
        writes.remove(name);
        for pending in &mut open {
            pending.tainted |= !pending.reads.is_disjoint(&writes);
        }
        if touches {
            let mut reads = HashSet::new();
            stmt_reads(stmt, &mut reads);
            let first = open.is_empty();
            open.push(Open {
                stmt,
                reads,
                tainted: false,
                first,
            });
        }
    }
    match open.is_empty() {
        true => pairs,
        false => Vec::new(),
    }
}
