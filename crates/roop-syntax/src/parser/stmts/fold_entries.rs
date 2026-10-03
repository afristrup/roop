use crate::{Attr, Block, Entry, Span, Stmt, StmtKind};

/// The statements of a block that ends at `end`, where each `ancilla x: T = e;`
/// becomes an ancilla whose body is everything after it.
pub fn fold_entries(entries: Vec<Entry>, end: usize) -> Vec<Stmt> {
    let mut rest: Vec<Stmt> = Vec::new();
    for entry in entries.into_iter().rev() {
        match entry {
            Entry::Stmt(stmt) => rest.insert(0, stmt),
            Entry::Declare {
                name,
                ty,
                init,
                auto,
                start,
                after,
            } => {
                let body = Block {
                    stmts: std::mem::take(&mut rest),
                    span: Span::from(after..end),
                };
                rest = vec![Stmt {
                    attrs: if auto { vec![Attr::Auto] } else { Vec::new() },
                    kind: StmtKind::Ancilla {
                        name,
                        ty,
                        init,
                        body,
                    },
                    span: Span::from(start..end),
                }];
            }
        }
    }
    rest
}
