use crate::is_task;
use roop_syntax::{Block, Stmt};

/// A block's statements, with each maximal run of adjacent `#[concurrent]`
/// statements gathered into one group of tasks.
pub enum TaskItem<'a> {
    Single(&'a Stmt),
    Group(Vec<&'a Stmt>),
}

pub fn task_groups(block: &Block) -> Vec<TaskItem<'_>> {
    let mut items = Vec::new();
    for stmt in &block.stmts {
        match (is_task(stmt), items.last_mut()) {
            (true, Some(TaskItem::Group(tasks))) => tasks.push(stmt),
            (true, _) => items.push(TaskItem::Group(vec![stmt])),
            (false, _) => items.push(TaskItem::Single(stmt)),
        }
    }
    items
}
