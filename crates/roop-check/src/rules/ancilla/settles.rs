use crate::{Mutability, stmt_writes};
use roop_syntax::{Block, Place, Stmt, StmtKind};
use std::collections::HashSet;

/// Whether the statement leaves the whole variable at zero whatever it held
/// before: it is a `keep` of it, or a loop whose body, and its step if that
/// writes it too, ends by keeping it.
pub fn settles(stmt: &Stmt, name: &str, mutability: &Mutability) -> bool {
    match &stmt.kind {
        StmtKind::Keep(Place::Var(kept)) => kept == name,
        StmtKind::Block(block) => ends_kept(block, name, mutability),
        StmtKind::From { body, step, .. } => {
            ends_kept(body, name, mutability)
                && (!writes(step, name, mutability) || ends_kept(step, name, mutability))
        }
        _ => false,
    }
}

fn ends_kept(block: &Block, name: &str, mutability: &Mutability) -> bool {
    block
        .stmts
        .iter()
        .rfind(|s| {
            let mut out = HashSet::new();
            stmt_writes(s, &mut out, Some(mutability));
            out.contains(name)
        })
        .is_some_and(|last| settles(last, name, mutability))
}

fn writes(block: &Block, name: &str, mutability: &Mutability) -> bool {
    let mut out = HashSet::new();
    block
        .stmts
        .iter()
        .for_each(|s| stmt_writes(s, &mut out, Some(mutability)));
    out.contains(name)
}
