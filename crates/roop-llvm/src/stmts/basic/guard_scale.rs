use crate::{CodegenError, FnGen, Kind, Value, bool_type, gen_assert, llvm_type};
use roop_syntax::UpdateOp;

/// Traps unless `old op factor` can be undone: the factor is nonzero and, for
/// integers, a product does not overflow or a quotient is exact. Written with
/// selects and a division by a safe divisor, so it needs no intrinsics and
/// runs on every target.
pub fn guard_scale(
    g: &mut FnGen,
    op: UpdateOp,
    kind: Kind,
    old: &Value,
    factor: &Value,
) -> Result<(), CodegenError> {
    let ty = llvm_type(g.ctx, &old.ty)?;
    let (x, e) = (old.reg.as_str(), factor.reg.as_str());
    let line = |g: &mut FnGen, what: &str, text: String| {
        let reg = format!("%{}", g.fresh(what));
        g.emit(&format!("{reg} = {text}"));
        reg
    };
    if kind == Kind::Float {
        let nonzero = line(g, "nz", format!("fcmp one {ty} {e}, 0.000000e+00"));
        return assert_ok(g, nonzero);
    }
    let zero = line(g, "zero", format!("icmp eq {ty} {e}, 0"));
    let minus_one = line(g, "m1", format!("icmp eq {ty} {e}, -1"));
    let x_min = line(g, "xmin", format!("icmp eq {ty} {x}, -9223372036854775808"));
    let special = line(g, "special", format!("or i1 {zero}, {minus_one}"));
    let divisor = line(
        g,
        "divisor",
        format!("select i1 {special}, {ty} 1, {ty} {e}"),
    );
    let overflow_m1 = line(g, "ovm1", format!("and i1 {minus_one}, {x_min}"));
    let bad = match op {
        UpdateOp::Mul => {
            let product = line(g, "prod", format!("mul {ty} {x}, {e}"));
            let back = line(g, "back", format!("sdiv {ty} {product}, {divisor}"));
            let changed = line(g, "changed", format!("icmp ne {ty} {back}, {x}"));
            let regular = line(g, "regular", format!("xor i1 {special}, true"));
            let overflow = line(g, "ovg", format!("and i1 {regular}, {changed}"));
            line(g, "bad", format!("or i1 {overflow}, {overflow_m1}"))
        }
        _ => {
            let rest = line(g, "rest", format!("srem {ty} {x}, {divisor}"));
            let inexact = line(g, "inexact", format!("icmp ne {ty} {rest}, 0"));
            line(g, "bad", format!("or i1 {inexact}, {overflow_m1}"))
        }
    };
    let nonzero = line(g, "nz", format!("xor i1 {zero}, true"));
    let fine = line(g, "fine", format!("xor i1 {bad}, true"));
    let ok = line(g, "ok", format!("and i1 {nonzero}, {fine}"));
    assert_ok(g, ok)
}

fn assert_ok(g: &mut FnGen, reg: String) -> Result<(), CodegenError> {
    gen_assert(
        g,
        &Value {
            reg,
            ty: bool_type(),
        },
    )
}
