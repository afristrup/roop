use crate::is_task;
use roop_syntax::{Block, Stmt};

/// A block's statements, with each maximal run of adjacent `#[concurrent]`
/// statements gathered into one group of tasks.
pub enum Item<'a> {
    Single(&'a Stmt),
    Group(Vec<&'a Stmt>),
}

pub fn task_groups(block: &Block) -> Vec<Item<'_>> {
    let mut items = Vec::new();
    for stmt in &block.stmts {
        match (is_task(stmt), items.last_mut()) {
            (true, Some(Item::Group(tasks))) => tasks.push(stmt),
            (true, _) => items.push(Item::Group(vec![stmt])),
            (false, _) => items.push(Item::Single(stmt)),
        }
    }
    items
}
