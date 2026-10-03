use roop_check::body_effects;
use roop_syntax::Block;

/// Whether the block reads or writes variable `name`.
pub fn mentions(block: &Block, name: &str) -> bool {
    let effects = body_effects(block);
    effects.reads.contains(name) || effects.writes.contains(name)
}
