use crate::{
    CodegenError, Dir, FnGen, gen_call_status, gen_stmt, gen_tagged_try, unwind_from, unwind_if,
    unwind_match, unwind_scope,
};
use roop_syntax::{Stmt, StmtKind};

/// One statement that, if it fails, has undone its own partial effects and
/// jumps to `fail`. Simple statements fail before they change anything.
pub fn gen_unwinding_stmt(
    g: &mut FnGen,
    stmt: &Stmt,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        StmtKind::Update { .. }
        | StmtKind::Swap(..)
        | StmtKind::Push { .. }
        | StmtKind::Pop { .. }
        | StmtKind::Overwrite { .. } => gen_stmt(g, stmt, dir),
        StmtKind::Call { callee, args } => gen_call_status(g, callee, args, false, dir, fail),
        StmtKind::Uncall { callee, args } => gen_call_status(g, callee, args, true, dir, fail),
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => unwind_if(g, cond, then_block, else_block, exit, dir, fail),
        StmtKind::Match { scrutinee, arms } => unwind_match(g, scrutinee, arms, dir, fail),
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => unwind_from(g, entry, body, step, until, dir, fail),
        StmtKind::Ancilla { .. }
        | StmtKind::Borrow { .. }
        | StmtKind::Block(_)
        | StmtKind::Logged { .. } => unwind_scope(g, stmt, dir, fail),
        StmtKind::Try {
            body,
            handler,
            outcome: Some(outcome),
        } => gen_tagged_try(g, body, handler, outcome, dir, fail),
        StmtKind::Try { .. }
        | StmtKind::Irrev(_)
        | StmtKind::Chan { .. }
        | StmtKind::Send { .. }
        | StmtKind::Recv { .. } => Err(CodegenError::Unsupported(
            "a statement a failed try could not undo",
        )),
    }
}
