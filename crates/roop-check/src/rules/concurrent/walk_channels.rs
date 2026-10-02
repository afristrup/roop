use crate::{CheckError, Item, analyse_group, child_blocks, task_groups};
use roop_syntax::{Block, StmtKind};

/// Walks a function body checking every task group, and that each channel
/// operation sits inside a task and names a channel that is in scope.
pub fn walk_channels(
    block: &Block,
    declared: &mut Vec<String>,
    in_task: bool,
) -> Result<(), CheckError> {
    for item in task_groups(block) {
        match item {
            Item::Group(tasks) => {
                analyse_group(&tasks)?;
                for task in tasks {
                    for child in child_blocks(task) {
                        walk_channels(child, declared, true)?;
                    }
                }
            }
            Item::Single(stmt) => match &stmt.kind {
                StmtKind::Send { chan, .. } | StmtKind::Recv { chan, .. } => {
                    if !declared.contains(chan) {
                        return Err(CheckError::UnknownChannel {
                            chan: chan.clone(),
                            span: stmt.span,
                        });
                    }
                    if !in_task {
                        return Err(CheckError::ChannelOpOutsideTask {
                            chan: chan.clone(),
                            span: stmt.span,
                        });
                    }
                }
                StmtKind::Chan { name, body, .. } => {
                    declared.push(name.clone());
                    walk_channels(body, declared, in_task)?;
                    declared.pop();
                }
                _ => {
                    for child in child_blocks(stmt) {
                        walk_channels(child, declared, in_task)?;
                    }
                }
            },
        }
    }
    Ok(())
}
