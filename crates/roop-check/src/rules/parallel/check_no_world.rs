use crate::{CheckError, find_world_call};
use roop_syntax::{Attr, Stmt};
use std::collections::HashSet;

use crate::child_blocks;

/// A parallel loop or a concurrent task runs in no fixed order, and the world
/// outside has one: what a program prints, reads or writes is in a sequence.
/// So neither may reach a function that changes the world.
pub fn check_no_world(stmt: &Stmt, world: &HashSet<&str>) -> Result<(), CheckError> {
    if stmt.attrs.is_empty()
        || !stmt
            .attrs
            .iter()
            .all(|a| matches!(a, Attr::Parallel { .. } | Attr::Concurrent))
    {
        return Ok(());
    }
    for block in child_blocks(stmt) {
        if let Some((callee, span)) = find_world_call(block, world) {
            return Err(CheckError::WorldInParallel { callee, span });
        }
    }
    Ok(())
}
