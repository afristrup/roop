use crate::{
    CodegenError, Dir, FnGen, ParallelLoop, Value, bool_type, gen_assert, gen_place,
    iteration_space, mem_load,
};
use roop_syntax::{Block, Expr, Place, counted_loop};

/// Shared by every target: recognize the loop, evaluate its bounds, trap
/// unless it terminates, and assert the entry condition.
pub fn parallel_prologue(
    g: &mut FnGen,
    entry: &Expr,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<ParallelLoop, CodegenError> {
    let counted = counted_loop(entry, step, until).ok_or(CodegenError::InvalidOperand(
        "parallel loop must be a counted loop",
    ))?;
    let space = iteration_space(g, &counted)?;
    let slot = gen_place(g, &Place::Var(counted.var.into()))?;
    let (start, end) = match dir {
        Dir::Forward => (space.lo.clone(), space.hi.clone()),
        Dir::Backward => (space.hi.clone(), space.lo.clone()),
    };
    let current = mem_load(g, &slot)?;
    let at_start = format!("%{}", g.fresh("t"));
    g.emit(&format!(
        "{at_start} = icmp eq i64 {}, {}",
        current.reg, start.reg
    ));
    gen_assert(
        g,
        &Value {
            reg: at_start,
            ty: bool_type(),
        },
    )?;
    Ok(ParallelLoop {
        var: counted.var.into(),
        slot,
        space,
        step: counted.step,
        end,
    })
}
