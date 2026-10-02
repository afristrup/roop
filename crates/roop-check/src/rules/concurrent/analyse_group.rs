use crate::{CheckError, channel_ops, check_task_disjointness, simulate_group};
use roop_syntax::{Block, Span, Stmt, StmtKind};

/// Checks a group of tasks as a closed system: disjoint state, a static
/// channel protocol, no deadlock, and every channel drained at the end.
pub fn analyse_group(tasks: &[&Stmt]) -> Result<(), CheckError> {
    let span = Span::from(tasks[0].span.start..tasks[tasks.len() - 1].span.end);
    let blocks: Vec<&Block> = tasks
        .iter()
        .map(|t| match &t.kind {
            StmtKind::Block(block) => Ok(block),
            _ => Err(CheckError::ConcurrentNotBlock { span: t.span }),
        })
        .collect::<Result<_, _>>()?;
    check_task_disjointness(&blocks, span)?;
    let ops = blocks
        .iter()
        .map(|block| channel_ops(block, &mut Vec::new()))
        .collect::<Result<Vec<_>, _>>()?;
    simulate_group(&ops, span)
}
