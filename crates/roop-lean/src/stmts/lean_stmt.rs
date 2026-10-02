use crate::{
    Ctx, Dir, Env, LeanError, Out, lean_ancilla, lean_block, lean_borrow, lean_call, lean_if,
    lean_logged, lean_loop, lean_match, lean_overwrite, lean_pop, lean_push, lean_swap, lean_try,
    lean_update,
};
use roop_syntax::{Attr, Stmt, StmtKind};

pub fn lean_stmt(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    stmt: &Stmt,
    dir: Dir,
) -> Result<(), LeanError> {
    if stmt.attrs.contains(&Attr::Concurrent) {
        return Err(LeanError::Unsupported("concurrent tasks".into()));
    }
    let unsupported = |what: &str| Err(LeanError::Unsupported(what.into()));
    match &stmt.kind {
        StmtKind::Update { target, op, value } => {
            lean_update(cx, env, out, target, *op, value, dir)
        }
        StmtKind::Swap(a, b) => lean_swap(cx, env, out, a, b),
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => lean_if(cx, env, out, cond, then_block, else_block, exit, dir),
        StmtKind::Match { scrutinee, arms } => lean_match(cx, env, out, scrutinee, arms, dir),
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => {
            let parallel = stmt
                .attrs
                .iter()
                .any(|a| matches!(a, Attr::Parallel { .. }));
            lean_loop(
                cx,
                env,
                out,
                stmt.span.start,
                parallel,
                entry,
                body,
                step,
                until,
                dir,
            )
        }
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => lean_ancilla(cx, env, out, name, ty, init, body, dir),
        StmtKind::Borrow { name, source, body } => {
            lean_borrow(cx, env, out, name, source, body, dir)
        }
        StmtKind::Call { callee, args } => lean_call(cx, env, out, callee, args, false, dir),
        StmtKind::Uncall { callee, args } => lean_call(cx, env, out, callee, args, true, dir),
        StmtKind::Block(body) => lean_block(cx, env, out, body, dir),
        StmtKind::Irrev(body) => match dir {
            Dir::Forward => lean_block(cx, env, out, body, dir),
            Dir::Backward => unsupported("an irrev block run backward"),
        },
        StmtKind::Push { stack, source } => lean_push(cx, env, out, stack, source, dir),
        StmtKind::Pop { stack, target } => lean_pop(cx, env, out, stack, target, dir),
        StmtKind::Logged { history, body } => lean_logged(cx, env, out, history, body, dir),
        StmtKind::Overwrite { target, op, value } => {
            lean_overwrite(cx, env, out, target, *op, value, dir)
        }
        StmtKind::Try {
            body,
            handler,
            outcome: Some(outcome),
        } => lean_try(cx, env, out, stmt.span.start, body, handler, outcome, dir),
        StmtKind::Try { .. } => unsupported("a try without an outcome"),
        StmtKind::Chan { .. } | StmtKind::Send { .. } | StmtKind::Recv { .. } => {
            unsupported("channels")
        }
    }
}
