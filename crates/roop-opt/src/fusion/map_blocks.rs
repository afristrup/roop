use roop_syntax::{Block, MatchArm, Stmt, StmtKind};

/// Rebuilds the statement with `f` applied to each directly nested block.
pub fn map_blocks(stmt: &Stmt, f: &impl Fn(&Block) -> Block) -> Stmt {
    let kind = match &stmt.kind {
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => StmtKind::If {
            cond: cond.clone(),
            then_block: f(then_block),
            else_block: f(else_block),
            exit: exit.clone(),
        },
        StmtKind::Match { scrutinee, arms } => StmtKind::Match {
            scrutinee: scrutinee.clone(),
            arms: arms
                .iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern.clone(),
                    body: f(&arm.body),
                    exit: arm.exit.clone(),
                })
                .collect(),
        },
        StmtKind::Try { body, handler } => StmtKind::Try {
            body: f(body),
            handler: f(handler),
        },
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => StmtKind::From {
            entry: entry.clone(),
            body: f(body),
            step: f(step),
            until: until.clone(),
        },
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => StmtKind::Ancilla {
            name: name.clone(),
            ty: ty.clone(),
            init: init.clone(),
            body: f(body),
        },
        StmtKind::Block(body) => StmtKind::Block(f(body)),
        StmtKind::Chan { name, ty, body } => StmtKind::Chan {
            name: name.clone(),
            ty: ty.clone(),
            body: f(body),
        },
        StmtKind::Borrow { name, source, body } => StmtKind::Borrow {
            name: name.clone(),
            source: source.clone(),
            body: f(body),
        },
        other => other.clone(),
    };
    Stmt {
        attrs: stmt.attrs.clone(),
        kind,
        span: stmt.span,
    }
}
