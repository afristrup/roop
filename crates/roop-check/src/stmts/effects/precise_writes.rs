use crate::{Mutability, body_effects, stmt_writes};
use roop_syntax::Block;
use std::collections::{BTreeSet, HashSet};

/// The variables a block writes, not counting what it only declares itself.
/// Unlike `body_effects`, a call writes only the arguments it passes for `&mut`
/// parameters, so a variable that is merely read by every call stays out.
pub fn precise_writes(block: &Block, mutability: &Mutability) -> BTreeSet<String> {
    let mut found: HashSet<&str> = HashSet::new();
    for stmt in &block.stmts {
        stmt_writes(stmt, &mut found, Some(mutability));
    }
    let outer = body_effects(block).writes;
    found
        .into_iter()
        .filter(|name| outer.contains(*name))
        .map(String::from)
        .collect()
}
