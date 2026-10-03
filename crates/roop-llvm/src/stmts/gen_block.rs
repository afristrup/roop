use crate::{CodegenError, Dir, FnGen, gen_group, gen_stmt};
use roop_check::{TaskItem, task_groups};
use roop_syntax::Block;

/// Backward execution runs the statements in reverse order. Adjacent
/// `#[concurrent]` blocks form one group, which runs as a single step.
pub fn gen_block(g: &mut FnGen, block: &Block, dir: Dir) -> Result<(), CodegenError> {
    let items = task_groups(block);
    let ordered: Vec<&TaskItem> = match dir {
        Dir::Forward => items.iter().collect(),
        Dir::Backward => items.iter().rev().collect(),
    };
    for item in ordered {
        match item {
            TaskItem::Single(stmt) => gen_stmt(g, stmt, dir)?,
            TaskItem::Group(tasks) => gen_group(g, tasks, dir)?,
        }
    }
    Ok(())
}
