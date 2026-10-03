use crate::{CodegenError, FnGen, Value, bool_type, gen_assert, gen_expr, int_op, same_type};
use roop_syntax::{CountedLoop, Type};

#[derive(Clone)]
pub struct IterationSpace {
    pub lo: Value,
    pub hi: Value,
    pub count: String,
}

/// Evaluates the bounds once and traps unless the loop would terminate:
/// `hi >= lo` and `hi - lo` divisible by the step.
pub fn iteration_space(
    g: &mut FnGen,
    counted: &CountedLoop,
) -> Result<IterationSpace, CodegenError> {
    let (lo, hi) = (gen_expr(g, counted.lo)?, gen_expr(g, counted.hi)?);
    let int = Type::Named("i64".into());
    same_type(&int, &lo.ty)?;
    same_type(&int, &hi.ty)?;
    let span = int_op(g, "sub", &hi.reg, &lo.reg);
    let ordered = format!("%{}", g.fresh("t"));
    g.emit(&format!("{ordered} = icmp sge i64 {}, {}", hi.reg, lo.reg));
    let rem = int_op(g, "srem", &span, &counted.step.to_string());
    let aligned = format!("%{}", g.fresh("t"));
    g.emit(&format!("{aligned} = icmp eq i64 {rem}, 0"));
    let ok = format!("%{}", g.fresh("t"));
    g.emit(&format!("{ok} = and i1 {ordered}, {aligned}"));
    gen_assert(
        g,
        &Value {
            reg: ok,
            ty: bool_type(),
        },
    )?;
    let quotient = int_op(g, "sdiv", &span, &counted.step.to_string());
    let count = int_op(g, "add", &quotient, "1");
    Ok(IterationSpace { lo, hi, count })
}
