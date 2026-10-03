use roop_syntax::{Block, Stmt, StmtKind};

/// Runs `f` on each block directly inside the statement.
pub fn blocks_mut<E>(
    stmt: &mut Stmt,
    f: &mut dyn FnMut(&mut Block) -> Result<(), E>,
) -> Result<(), E> {
    match &mut stmt.kind {
        StmtKind::If {
            then_block,
            else_block,
            ..
        } => {
            f(then_block)?;
            f(else_block)
        }
        StmtKind::From { body, step, .. } => {
            f(body)?;
            f(step)
        }
        StmtKind::Match { arms, .. } => arms.iter_mut().try_for_each(|arm| f(&mut arm.body)),
        StmtKind::Try { body, handler, .. } => {
            f(body)?;
            f(handler)
        }
        StmtKind::Ancilla { body, .. }
        | StmtKind::Borrow { body, .. }
        | StmtKind::Chan { body, .. }
        | StmtKind::Logged { body, .. }
        | StmtKind::Block(body)
        | StmtKind::Irrev(body) => f(body),
        _ => Ok(()),
    }
}
