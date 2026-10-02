use roop_syntax::{Block, StmtKind};

/// Whether any ancilla or borrow in the block binds `name`.
pub fn declares(block: &Block, name: &str) -> bool {
    block.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Block(body) | StmtKind::Irrev(body) | StmtKind::Chan { body, .. } => {
            declares(body, name)
        }
        StmtKind::Ancilla { name: n, body, .. } | StmtKind::Borrow { name: n, body, .. } => {
            n == name || declares(body, name)
        }
        StmtKind::If {
            then_block,
            else_block,
            ..
        } => declares(then_block, name) || declares(else_block, name),
        StmtKind::From { body, step, .. } => declares(body, name) || declares(step, name),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| declares(&a.body, name)),
        StmtKind::Try { body, handler, .. } => declares(body, name) || declares(handler, name),
        StmtKind::Logged { body, .. } => declares(body, name),
        _ => false,
    })
}
