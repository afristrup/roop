use crate::{Binding, collect_accesses, place_root};
use roop_syntax::Block;
use std::collections::BTreeSet;

/// Variables a block reads and writes, ignoring iteration-private locals.
/// A variable in both sets is "inout" in the RBLAS sense, in only `reads` it
/// is a read-only input that needs no copy back.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BodyEffects {
    pub reads: BTreeSet<String>,
    pub writes: BTreeSet<String>,
}

pub fn body_effects(block: &Block) -> BodyEffects {
    let mut accesses = Vec::new();
    collect_accesses(block, &mut Vec::<Binding>::new(), &mut accesses, None);
    let mut effects = BodyEffects::default();
    for access in accesses.iter().filter(|a| !a.local) {
        let root = place_root(&access.place).to_string();
        if access.write {
            effects.writes.insert(root);
        } else {
            effects.reads.insert(root);
        }
    }
    effects
}
