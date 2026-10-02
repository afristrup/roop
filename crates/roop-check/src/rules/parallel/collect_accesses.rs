use crate::{
    Access, Binding, expr_places, place_index_reads, push_access, resolve_place, stmt_exprs,
};
use roop_syntax::{Block, Expr, Place, Stmt, StmtKind};

/// Every access in the block. Calls are assumed to write each place argument.
pub fn collect_accesses(block: &Block, scope: &mut Vec<Binding>, out: &mut Vec<Access>) {
    for stmt in &block.stmts {
        collect_stmt(stmt, scope, out);
    }
}

fn collect_stmt(stmt: &Stmt, scope: &mut Vec<Binding>, out: &mut Vec<Access>) {
    let mut reads: Vec<&Place> = Vec::new();
    for expr in stmt_exprs(stmt) {
        expr_places(expr, &mut reads);
    }
    match &stmt.kind {
        StmtKind::Update { target, .. } => written(target, scope, out),
        StmtKind::Send { source: place, .. } | StmtKind::Recv { target: place, .. } => {
            written(place, scope, out)
        }
        StmtKind::Swap(a, b) => {
            written(a, scope, out);
            written(b, scope, out);
        }
        StmtKind::Call { args, .. } | StmtKind::Uncall { args, .. } => {
            reads.clear();
            for arg in args {
                match arg {
                    Expr::Place(place) => written(place, scope, out),
                    other => expr_places(other, &mut reads),
                }
            }
        }
        _ => {}
    }
    for place in reads {
        push_access(place, false, scope, out);
    }
    nested(stmt, scope, out);
}

fn written(place: &Place, scope: &[Binding], out: &mut Vec<Access>) {
    push_access(place, true, scope, out);
    let mut index_reads = Vec::new();
    place_index_reads(place, &mut index_reads);
    for read in index_reads {
        push_access(read, false, scope, out);
    }
}

fn nested(stmt: &Stmt, scope: &mut Vec<Binding>, out: &mut Vec<Access>) {
    match &stmt.kind {
        StmtKind::Ancilla { name, body, .. } => {
            scope.push(Binding::Local(name.clone()));
            collect_accesses(body, scope, out);
            scope.pop();
        }
        StmtKind::Borrow { name, source, body } => {
            let (resolved, _) = resolve_place(source, scope);
            scope.push(Binding::Alias(name.clone(), resolved));
            collect_accesses(body, scope, out);
            scope.pop();
        }
        _ => {
            for child in crate::child_blocks(stmt) {
                collect_accesses(child, scope, out);
            }
        }
    }
}
