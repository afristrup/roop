use roop_syntax::{Span, Stmt, StmtKind};

/// A statement with no attributes and no position.
pub fn einsum_stmt(kind: StmtKind) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind,
        span: Span::from(0..0),
    }
}
