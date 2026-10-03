use crate::{rename_expr, rename_place};
use roop_syntax::{Block, MatchArm, Stmt, StmtKind};

/// Renames every use of variable `from` to `to`. Callers make sure no block
/// declares a binding that shadows either name.
pub fn rename_block(block: &Block, from: &str, to: &str) -> Block {
    Block {
        stmts: block
            .stmts
            .iter()
            .map(|s| rename_stmt(s, from, to))
            .collect(),
        span: block.span,
    }
}

fn rename_stmt(stmt: &Stmt, from: &str, to: &str) -> Stmt {
    let e = |x| rename_expr(x, from, to);
    let p = |x| rename_place(x, from, to);
    let b = |x| rename_block(x, from, to);
    let kind = match &stmt.kind {
        StmtKind::Update { target, op, value } => StmtKind::Update {
            target: p(target),
            op: *op,
            value: e(value),
        },
        StmtKind::Swap(x, y) => StmtKind::Swap(p(x), p(y)),
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => StmtKind::If {
            cond: e(cond),
            then_block: b(then_block),
            else_block: b(else_block),
            exit: e(exit),
        },
        StmtKind::Match { scrutinee, arms } => StmtKind::Match {
            scrutinee: e(scrutinee),
            arms: arms
                .iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern.clone(),
                    body: b(&arm.body),
                    exit: e(&arm.exit),
                })
                .collect(),
        },
        StmtKind::Try {
            body,
            handler,
            outcome,
        } => StmtKind::Try {
            body: b(body),
            handler: b(handler),
            outcome: outcome.as_ref().map(p),
        },
        StmtKind::Logged { history, body } => StmtKind::Logged {
            history: p(history),
            body: b(body),
        },
        StmtKind::Push { stack, source } => StmtKind::Push {
            stack: p(stack),
            source: p(source),
        },
        StmtKind::Pop { stack, target } => StmtKind::Pop {
            stack: p(stack),
            target: p(target),
        },
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => StmtKind::From {
            entry: e(entry),
            body: b(body),
            step: b(step),
            until: e(until),
        },
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => StmtKind::Ancilla {
            name: name.clone(),
            ty: ty.clone(),
            init: e(init),
            body: b(body),
        },
        StmtKind::Borrow { name, source, body } => StmtKind::Borrow {
            name: name.clone(),
            source: p(source),
            body: b(body),
        },
        StmtKind::Block(body) => StmtKind::Block(b(body)),
        StmtKind::Irrev(body) => StmtKind::Irrev(b(body)),
        StmtKind::Overwrite { target, op, value } => StmtKind::Overwrite {
            target: p(target),
            op: *op,
            value: e(value),
        },
        StmtKind::Chan { name, ty, body } => StmtKind::Chan {
            name: name.clone(),
            ty: ty.clone(),
            body: b(body),
        },
        StmtKind::Send { chan, source } => StmtKind::Send {
            chan: chan.clone(),
            source: p(source),
        },
        StmtKind::Recv { chan, target } => StmtKind::Recv {
            chan: chan.clone(),
            target: p(target),
        },
        StmtKind::Call {
            callee,
            generics,
            args,
        } => StmtKind::Call {
            callee: callee.clone(),
            generics: generics.clone(),
            args: args.iter().map(e).collect(),
        },
        StmtKind::Uncall {
            callee,
            generics,
            args,
        } => StmtKind::Uncall {
            callee: callee.clone(),
            generics: generics.clone(),
            args: args.iter().map(e).collect(),
        },
    };
    Stmt {
        attrs: stmt.attrs.clone(),
        kind,
        span: stmt.span,
    }
}
