use super::{CheckError, check_ancilla, check_match, check_update};
use roop_syntax::{Block, StmtKind};

pub fn check_block(block: &Block) -> Result<(), CheckError> {
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Update { target, value, .. } => check_update(target, value, stmt.span)?,
            StmtKind::Swap(..) | StmtKind::Call { .. } | StmtKind::Uncall { .. } => {}
            StmtKind::If {
                then_block,
                else_block,
                ..
            } => {
                check_block(then_block)?;
                check_block(else_block)?;
            }
            StmtKind::Match { arms, .. } => check_match(arms, stmt.span)?,
            StmtKind::From { body, step, .. } => {
                check_block(body)?;
                check_block(step)?;
            }
            StmtKind::Ancilla { name, body, .. } => {
                check_ancilla(name, body, stmt.span)?;
                check_block(body)?;
            }
        }
    }
    Ok(())
}
