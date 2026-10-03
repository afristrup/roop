use crate::{Mutability, place_root};
use roop_syntax::{Block, Expr, Stmt, StmtKind};
use std::collections::HashSet;

/// The variables a statement may write. A call writes the arguments it passes
/// for `&mut` parameters, or all of them when `mutability` does not know the
/// callee.
pub fn stmt_writes<'a>(
    stmt: &'a Stmt,
    out: &mut HashSet<&'a str>,
    mutability: Option<&Mutability>,
) {
    match &stmt.kind {
        StmtKind::Update { target, .. } | StmtKind::Overwrite { target, .. } => {
            out.insert(place_root(target));
        }
        StmtKind::Swap(a, b) => {
            out.insert(place_root(a));
            out.insert(place_root(b));
        }
        StmtKind::Send { source: p, .. } | StmtKind::Recv { target: p, .. } => {
            out.insert(place_root(p));
        }
        StmtKind::Push { stack, source } => {
            out.insert(place_root(stack));
            out.insert(place_root(source));
        }
        StmtKind::Pop { stack, target } => {
            out.insert(place_root(stack));
            out.insert(place_root(target));
        }
        StmtKind::Keep(place) => {
            out.insert(place_root(place));
        }
        StmtKind::Logged { history, body } => {
            out.insert(place_root(history));
            block_writes(body, out, mutability);
        }
        StmtKind::Block(block) | StmtKind::Irrev(block) | StmtKind::Chan { body: block, .. } => {
            block_writes(block, out, mutability)
        }
        StmtKind::Call { callee, args, .. } | StmtKind::Uncall { callee, args, .. } => {
            let flags = mutability.and_then(|m| m.get(callee.as_str()));
            for (k, arg) in args.iter().enumerate() {
                let written = flags.is_none_or(|f| f.get(k).copied().unwrap_or(true));
                if let (true, Expr::Place(place)) = (written, arg) {
                    out.insert(place_root(place));
                }
            }
        }
        StmtKind::If {
            then_block,
            else_block,
            ..
        } => {
            block_writes(then_block, out, mutability);
            block_writes(else_block, out, mutability);
        }
        StmtKind::Borrow { name, source, body } => {
            let mut inner = HashSet::new();
            block_writes(body, &mut inner, mutability);
            if inner.remove(name.as_str()) {
                out.insert(place_root(source));
            }
            out.extend(inner);
        }
        StmtKind::Try {
            body,
            handler,
            outcome,
        } => {
            block_writes(body, out, mutability);
            block_writes(handler, out, mutability);
            if let Some(outcome) = outcome {
                out.insert(place_root(outcome));
            }
        }
        StmtKind::Match { arms, .. } => {
            for arm in arms {
                block_writes(&arm.body, out, mutability);
            }
        }
        StmtKind::From { body, step, .. } => {
            block_writes(body, out, mutability);
            block_writes(step, out, mutability);
        }
        StmtKind::Ancilla { body, .. } => block_writes(body, out, mutability),
    }
}

fn block_writes<'a>(block: &'a Block, out: &mut HashSet<&'a str>, mutability: Option<&Mutability>) {
    for stmt in &block.stmts {
        stmt_writes(stmt, out, mutability);
    }
}
