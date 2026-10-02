use crate::place_root;
use roop_syntax::{Block, Expr, Stmt, StmtKind};
use std::collections::HashSet;

pub fn stmt_writes<'a>(stmt: &'a Stmt, out: &mut HashSet<&'a str>) {
    match &stmt.kind {
        StmtKind::Update { target, .. } => {
            out.insert(place_root(target));
        }
        StmtKind::Swap(a, b) => {
            out.insert(place_root(a));
            out.insert(place_root(b));
        }
        StmtKind::Send { source: p, .. } | StmtKind::Recv { target: p, .. } => {
            out.insert(place_root(p));
        }
        StmtKind::Block(block) | StmtKind::Chan { body: block, .. } => block_writes(block, out),
        StmtKind::Call { args, .. } | StmtKind::Uncall { args, .. } => {
            for arg in args {
                if let Expr::Place(place) = arg {
                    out.insert(place_root(place));
                }
            }
        }
        StmtKind::If {
            then_block,
            else_block,
            ..
        } => {
            block_writes(then_block, out);
            block_writes(else_block, out);
        }
        StmtKind::Borrow { name, source, body } => {
            let mut inner = HashSet::new();
            block_writes(body, &mut inner);
            if inner.remove(name.as_str()) {
                out.insert(place_root(source));
            }
            out.extend(inner);
        }
        StmtKind::Try { body, handler } => {
            block_writes(body, out);
            block_writes(handler, out);
        }
        StmtKind::Match { arms, .. } => {
            for arm in arms {
                block_writes(&arm.body, out);
            }
        }
        StmtKind::From { body, step, .. } => {
            block_writes(body, out);
            block_writes(step, out);
        }
        StmtKind::Ancilla { body, .. } => block_writes(body, out),
    }
}

fn block_writes<'a>(block: &'a Block, out: &mut HashSet<&'a str>) {
    for stmt in &block.stmts {
        stmt_writes(stmt, out);
    }
}
