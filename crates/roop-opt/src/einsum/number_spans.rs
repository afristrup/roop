use roop_syntax::{Span, Stmt, StmtKind};

/// Gives each generated statement its own position, in order. The Lean model
/// names a lifted loop after where it starts, so loops that all start at 0 would
/// share one name.
pub fn number_spans(stmts: &mut [Stmt], next: &mut usize) {
    for stmt in stmts {
        stmt.span = Span::from(*next..*next);
        *next += 1;
        match &mut stmt.kind {
            StmtKind::Ancilla { body, .. } => number_spans(&mut body.stmts, next),
            StmtKind::From { body, step, .. } => {
                number_spans(&mut body.stmts, next);
                number_spans(&mut step.stmts, next);
            }
            _ => {}
        }
    }
}
