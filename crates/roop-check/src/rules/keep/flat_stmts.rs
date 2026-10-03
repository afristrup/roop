use roop_syntax::{Block, Stmt, StmtKind};

/// The statements of a block in order, looking through ancilla and plain
/// blocks, which only scope names and always run.
pub fn flat_stmts(block: &Block) -> Vec<&Stmt> {
    let mut out = Vec::new();
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Ancilla { body, .. } | StmtKind::Block(body) => out.extend(flat_stmts(body)),
            _ => out.push(stmt),
        }
    }
    out
}
