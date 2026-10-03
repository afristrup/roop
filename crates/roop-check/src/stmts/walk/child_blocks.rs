use roop_syntax::{Block, Stmt, StmtKind};

pub fn child_blocks(stmt: &Stmt) -> Vec<&Block> {
    match &stmt.kind {
        StmtKind::If {
            then_block,
            else_block,
            ..
        } => vec![then_block, else_block],
        StmtKind::From { body, step, .. } => vec![body, step],
        StmtKind::Match { arms, .. } => arms.iter().map(|arm| &arm.body).collect(),
        StmtKind::Try { body, handler, .. } => vec![body, handler],
        StmtKind::Ancilla { body, .. }
        | StmtKind::Borrow { body, .. }
        | StmtKind::Chan { body, .. } => vec![body],
        StmtKind::Block(block) | StmtKind::Irrev(block) => vec![block],
        StmtKind::Logged { body, .. } => vec![body],
        StmtKind::Update { .. }
        | StmtKind::Swap(..)
        | StmtKind::Call { .. }
        | StmtKind::Uncall { .. }
        | StmtKind::Send { .. }
        | StmtKind::Recv { .. }
        | StmtKind::Push { .. }
        | StmtKind::Pop { .. }
        | StmtKind::Keep(_)
        | StmtKind::Overwrite { .. } => Vec::new(),
    }
}
