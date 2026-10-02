use roop_syntax::{Span, Stmt, StmtKind};

pub fn synth(kind: StmtKind, span: Span) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind,
        span,
    }
}
