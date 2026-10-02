use crate::{
    CodegenError, Dir, FnGen, Value, bool_type, capture_env, gen_assert, gen_place, int_op,
    iteration_space, llvm_type, outline_body,
};
use roop_syntax::{Block, Expr, Place, Type, counted_loop};

/// Runs the iterations of a counted loop on CPU threads. Forward ends with
/// the induction variable at `hi`; backward ends with it at `lo`.
pub fn gen_parallel_cpu(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let counted = counted_loop(entry, step, until)
        .ok_or(CodegenError::InvalidOperand("parallel loop must be a counted loop"))?;
    let space = iteration_space(g, &counted)?;
    let var = gen_place(g, &Place::Var(counted.var.into()))?;
    let (start, end) = match dir {
        Dir::Forward => (&space.lo, &space.hi),
        Dir::Backward => (&space.hi, &space.lo),
    };
    let ty = llvm_type(g.ctx, &Type::Named("i64".into()))?;
    let current = format!("%{}", g.fresh("t"));
    g.emit(&format!("{current} = load {ty}, ptr {}", var.addr));
    let at_start = format!("%{}", g.fresh("t"));
    g.emit(&format!("{at_start} = icmp eq i64 {current}, {}", start.reg));
    gen_assert(g, &Value { reg: at_start, ty: bool_type() })?;

    let env = capture_env(g);
    let symbol = outline_body(g, counted.var, body, dir)?;
    g.emit(&format!(
        "call void @roop_parallel_for(i64 {}, i64 {}, i64 {}, ptr @{symbol}, ptr {env})",
        space.lo.reg, space.count, counted.step
    ));
    g.emit(&format!("store i64 {}, ptr {}", end.reg, var.addr));
    let _ = int_op;
    Ok(())
}
