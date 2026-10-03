use crate::{Dir, FnGen, Slot};
use roop_syntax::Stmt;

/// The ancilla that a statement compiles to zeroing, when it is one of the pair
/// that `gen_ancilla` found and is compiled in the direction it chose.
pub fn cleared_slot(g: &FnGen, stmt: &Stmt, dir: Dir) -> Option<Slot> {
    let address = std::ptr::from_ref(stmt) as usize;
    g.clears
        .iter()
        .find(|c| c.stmt == address && c.dir == dir)
        .map(|c| c.slot.clone())
}
