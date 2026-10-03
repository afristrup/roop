use crate::{Scope, infer_block, infer_call, place_type};
use roop_syntax::{MatchArm, Stmt, StmtKind, Type};

/// The statement with the lengths filled in for the calls inside it that
/// leave them out.
pub fn infer_stmt(scope: &mut Scope, stmt: &Stmt) -> Stmt {
    let kind = match &stmt.kind {
        StmtKind::Call {
            callee,
            generics,
            args,
        } => StmtKind::Call {
            callee: callee.clone(),
            generics: infer_call(scope, callee, generics, args).unwrap_or_else(|| generics.clone()),
            args: args.clone(),
        },
        StmtKind::Uncall {
            callee,
            generics,
            args,
        } => StmtKind::Uncall {
            callee: callee.clone(),
            generics: infer_call(scope, callee, generics, args).unwrap_or_else(|| generics.clone()),
            args: args.clone(),
        },
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => StmtKind::If {
            cond: cond.clone(),
            then_block: infer_block(scope, then_block),
            else_block: infer_block(scope, else_block),
            exit: exit.clone(),
        },
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => StmtKind::From {
            entry: entry.clone(),
            body: infer_block(scope, body),
            step: infer_block(scope, step),
            until: until.clone(),
        },
        StmtKind::Match { scrutinee, arms } => StmtKind::Match {
            scrutinee: scrutinee.clone(),
            arms: arms
                .iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern.clone(),
                    body: infer_block(scope, &arm.body),
                    exit: arm.exit.clone(),
                })
                .collect(),
        },
        StmtKind::Try {
            body,
            handler,
            outcome,
        } => StmtKind::Try {
            body: infer_block(scope, body),
            handler: infer_block(scope, handler),
            outcome: outcome.clone(),
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
            body: within(scope, name, Some(ty.clone()), body),
        },
        StmtKind::Borrow { name, source, body } => StmtKind::Borrow {
            name: name.clone(),
            source: source.clone(),
            body: within(scope, name, place_type(scope, source), body),
        },
        StmtKind::Chan { name, ty, body } => StmtKind::Chan {
            name: name.clone(),
            ty: ty.clone(),
            body: infer_block(scope, body),
        },
        StmtKind::Block(body) => StmtKind::Block(infer_block(scope, body)),
        StmtKind::Irrev(body) => StmtKind::Irrev(infer_block(scope, body)),
        StmtKind::Logged { history, body } => StmtKind::Logged {
            history: history.clone(),
            body: infer_block(scope, body),
        },
        other => other.clone(),
    };
    Stmt {
        attrs: stmt.attrs.clone(),
        kind,
        span: stmt.span,
    }
}

/// A block in which `name` has type `ty`, when that is known.
fn within(
    scope: &mut Scope,
    name: &str,
    ty: Option<Type>,
    body: &roop_syntax::Block,
) -> roop_syntax::Block {
    let mark = scope.vars.len();
    if let Some(ty) = ty {
        scope.vars.push((name.to_string(), ty));
    }
    let out = infer_block(scope, body);
    scope.vars.truncate(mark);
    out
}
