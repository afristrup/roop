use super::{expr_vars, place_vars};
use roop_syntax::{Stmt, StmtKind};
use std::collections::HashSet;

/// Variables an invertible straight-line statement depends on.
pub fn stmt_reads<'a>(stmt: &'a Stmt, out: &mut HashSet<&'a str>) {
    match &stmt.kind {
        StmtKind::Update { target, value, .. } => {
            place_vars(target, out);
            expr_vars(value, out);
        }
        StmtKind::Swap(a, b) => {
            place_vars(a, out);
            place_vars(b, out);
        }
        StmtKind::Call { args, .. } | StmtKind::Uncall { args, .. } => {
            for arg in args {
                expr_vars(arg, out);
            }
        }
        _ => unreachable!("only straight-line statements are tracked"),
    }
}
