use roop_syntax::{Place, Span, Stmt, StmtKind};

/// `keep name;`, placed at `at`.
pub fn keep_stmt(name: &str, at: usize) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind: StmtKind::Keep(Place::Var(name.to_string())),
        span: Span::from(at..at),
    }
}
