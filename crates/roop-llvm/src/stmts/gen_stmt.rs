use crate::{
    CodegenError, Dialect, Dir, FnGen, gen_ancilla, gen_borrow, gen_call, gen_from, gen_if,
    gen_match, gen_parallel_auto, gen_parallel_cpu, gen_parallel_gpu, gen_swap, gen_update,
    parallel_attr,
};
use roop_syntax::{Stmt, StmtKind, Target};

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
        } => match parallel_attr(stmt).filter(|_| g.dialect == Dialect::Host) {
            None => gen_from(g, entry, body, step, until, dir),
            Some(None) => gen_parallel_auto(g, entry, body, step, until, dir),
            Some(Some(target)) => {
                if !g.ctx.options.parallel.allowed.contains(&target) {
                    return Err(CodegenError::Unsupported(
                        "a parallel target disabled in Roop.toml",
                    ));
                }
                match target {
                    Target::Cpu => gen_parallel_cpu(g, entry, body, step, until, dir),
                    Target::Nvptx => {
                        gen_parallel_gpu(g, Dialect::Nvptx, entry, body, step, until, dir)
                    }
                    Target::Metal => {
                        gen_parallel_gpu(g, Dialect::Air, entry, body, step, until, dir)
                    }
                }
            }
        },
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
