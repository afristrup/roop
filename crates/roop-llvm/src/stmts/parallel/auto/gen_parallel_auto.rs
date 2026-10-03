use crate::{
    Choice, CodegenError, Dialect, Dir, FnGen, choose_target, device_buffers, gen_from,
    gen_parallel_cpu, gen_parallel_gpu, has_f64, launch_cpu, launch_gpu, layout, mem_store,
    parallel_prologue, serial_is_faster, static_trip,
};
use roop_check::{body_effects, body_features};
use roop_syntax::{Block, Expr, Target, counted_loop};

/// A bare `#[parallel]`: estimate CPU against each permitted GPU and run the
/// loop wherever it should be faster.
pub fn gen_parallel_auto(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let counted = counted_loop(entry, step, until).ok_or(CodegenError::InvalidOperand(
        "parallel loop must be a counted loop",
    ))?;
    let features = body_features(body);
    let effects = body_effects(body);
    let buffers = device_buffers(g, counted.var, &effects);
    let mut bytes = 0;
    let mut uses_f64 = features.has_float_literal;
    for (_, slot, writable) in &buffers {
        bytes += layout(g.ctx, &slot.ty)?.0 * (1 + u64::from(*writable));
        uses_f64 |= has_f64(g.ctx, &slot.ty);
    }
    let options = &g.ctx.options.parallel;
    if let Some(trip) = static_trip(&counted)
        && !features.has_call
        && serial_is_faster(&options.cost, &features, trip)
    {
        return gen_from(g, entry, body, step, until, dir);
    }
    let gpu = options.auto_gpus.iter().copied().find(|t| match t {
        Target::Metal => !features.has_call && !uses_f64,
        Target::Nvptx => !features.has_call,
        Target::Cpu => false,
    });
    let choice = match gpu {
        Some(gpu) => choose_target(&options.cost, &features, bytes, static_trip(&counted), gpu),
        None => Choice::Cpu,
    };
    let dialect = |t| {
        if t == Target::Metal {
            Dialect::Air
        } else {
            Dialect::Nvptx
        }
    };
    match choice {
        Choice::Cpu => gen_parallel_cpu(g, entry, body, step, until, dir),
        Choice::Gpu(t) => gen_parallel_gpu(g, dialect(t), entry, body, step, until, dir),
        Choice::Runtime { gpu, break_even } => {
            let pl = parallel_prologue(g, entry, step, until, dir)?;
            let big = format!("%{}", g.fresh("t"));
            g.emit(&format!(
                "{big} = icmp sge i64 {}, {break_even}",
                pl.space.count
            ));
            let (on_gpu, on_cpu, done) = (g.fresh("L"), g.fresh("L"), g.fresh("L"));
            g.emit(&format!("br i1 {big}, label %{on_gpu}, label %{on_cpu}"));
            g.label(&on_gpu);
            launch_gpu(g, dialect(gpu), &pl, body, dir)?;
            g.emit(&format!("br label %{done}"));
            g.label(&on_cpu);
            launch_cpu(g, &pl, body, dir)?;
            g.emit(&format!("br label %{done}"));
            g.label(&done);
            mem_store(g, &pl.slot, &pl.end.reg)
        }
    }
}
