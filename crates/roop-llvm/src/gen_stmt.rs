use super::{
    CodegenError, Dir, FnGen, gen_ancilla, gen_borrow, gen_call, gen_from, gen_if, gen_match,
    gen_swap, gen_update,
};
use roop_syntax::{Stmt, StmtKind};

pub fn gen_stmt(g: &mut FnGen, stmt: &Stmt, dir: Dir) -> Result<(), CodegenError> {
    match &stmt.kind {
        StmtKind::Update { target, op, value } => gen_update(g, target, *op, value, dir),
        StmtKind::Swap(a, b) => gen_swap(g, a, b),
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => gen_if(g, cond, then_block, else_block, exit, dir),
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => gen_from(g, entry, body, step, until, dir),
        StmtKind::Match { scrutinee, arms } => gen_match(g, scrutinee, arms, dir),
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => gen_ancilla(g, name, ty, init, body, dir),
        StmtKind::Borrow { name, source, body } => gen_borrow(g, name, source, body, dir),
        StmtKind::Call { callee, args } => gen_call(g, callee, args, false, dir),
        StmtKind::Uncall { callee, args } => gen_call(g, callee, args, true, dir),
        StmtKind::Try { .. } => Err(CodegenError::Unsupported("try ... catch_rollback")),
    }
}
