use crate::{
    Access, Binding, Mutability, expr_places, place_index_reads, push_access, resolve_place,
    stmt_exprs,
};
use roop_syntax::{Block, Expr, Place, Stmt, StmtKind};

/// Every access in the block. A call is assumed to write each place argument,
/// unless `mutability` says which parameters of the callee it may write.
pub fn collect_accesses(
    block: &Block,
    scope: &mut Vec<Binding>,
    out: &mut Vec<Access>,
    mutability: Option<&Mutability>,
) {
    for stmt in &block.stmts {
        collect_stmt(stmt, scope, out, mutability);
    }
}

fn collect_stmt(
    stmt: &Stmt,
    scope: &mut Vec<Binding>,
    out: &mut Vec<Access>,
    mutability: Option<&Mutability>,
) {
    let mut reads: Vec<&Place> = Vec::new();
    for expr in stmt_exprs(stmt) {
        expr_places(expr, &mut reads);
    }
    match &stmt.kind {
        StmtKind::Update { target, .. } | StmtKind::Overwrite { target, .. } => {
            written(target, scope, out)
        }
        StmtKind::Send { source: place, .. } | StmtKind::Recv { target: place, .. } => {
            written(place, scope, out)
        }
        StmtKind::Push {
            stack,
            source: place,
        }
        | StmtKind::Pop {
            stack,
            target: place,
        } => {
            written(stack, scope, out);
            written(place, scope, out);
        }
        StmtKind::Logged { history, .. } => written(history, scope, out),
        StmtKind::Swap(a, b) => {
            written(a, scope, out);
            written(b, scope, out);
        }
        StmtKind::Call { callee, args, .. } | StmtKind::Uncall { callee, args, .. } => {
            reads.clear();
            let writable = mutability.and_then(|m| m.get(callee.as_str()));
            for (k, arg) in args.iter().enumerate() {
                let may_write = writable.is_none_or(|w| w.get(k).copied().unwrap_or(true));
                match arg {
                    Expr::Place(place) if may_write => written(place, scope, out),
                    other => expr_places(other, &mut reads),
                }
            }
        }
        _ => {}
    }
    for place in reads {
        push_access(place, false, scope, out);
    }
    nested(stmt, scope, out, mutability);
}

fn written(place: &Place, scope: &[Binding], out: &mut Vec<Access>) {
    push_access(place, true, scope, out);
    let mut index_reads = Vec::new();
    place_index_reads(place, &mut index_reads);
    for read in index_reads {
        push_access(read, false, scope, out);
    }
}

fn nested(
    stmt: &Stmt,
    scope: &mut Vec<Binding>,
    out: &mut Vec<Access>,
    mutability: Option<&Mutability>,
) {
    match &stmt.kind {
        StmtKind::Ancilla { name, body, .. } => {
            scope.push(Binding::Local(name.clone()));
            collect_accesses(body, scope, out, mutability);
            scope.pop();
        }
        StmtKind::Borrow { name, source, body } => {
            let (resolved, _) = resolve_place(source, scope);
            scope.push(Binding::Alias(name.clone(), resolved));
            collect_accesses(body, scope, out, mutability);
            scope.pop();
        }
        _ => {
            for child in crate::child_blocks(stmt) {
                collect_accesses(child, scope, out, mutability);
            }
        }
    }
}
