use crate::{Dialect, FnGen};

/// Writes `reg = instr ty lhs, rhs`. With overflow checks on, an `i64` add,
/// subtract or multiply on the host uses the overflow intrinsic instead and
/// raises the sticky `@roop_overflow` flag when it wraps; the result is the same
/// wrapped number either way.
pub fn emit_int_arith(g: &mut FnGen, reg: &str, instr: &str, ty: &str, lhs: &str, rhs: &str) {
    let intrinsic = match instr {
        "add" => "sadd",
        "sub" => "ssub",
        "mul" => "smul",
        _ => "",
    };
    let checked = g.ctx.options.check_overflow
        && g.dialect == Dialect::Host
        && ty == "i64"
        && !intrinsic.is_empty();
    if !checked {
        g.emit(&format!("{reg} = {instr} {ty} {lhs}, {rhs}"));
        return;
    }
    let pair = format!("%{}", g.fresh("ov"));
    let wrapped = format!("%{}", g.fresh("wrap"));
    let rare = format!("%{}", g.fresh("rare"));
    let (set, done) = (g.fresh("ovset"), g.fresh("ovdone"));
    g.emit(&format!(
        "{pair} = call {{ i64, i1 }} @llvm.{intrinsic}.with.overflow.i64(i64 {lhs}, i64 {rhs})"
    ));
    g.emit(&format!("{reg} = extractvalue {{ i64, i1 }} {pair}, 0"));
    g.emit(&format!("{wrapped} = extractvalue {{ i64, i1 }} {pair}, 1"));
    g.emit(&format!(
        "{rare} = call i1 @llvm.expect.i1(i1 {wrapped}, i1 false)"
    ));
    g.emit(&format!("br i1 {rare}, label %{set}, label %{done}"));
    g.label(&set);
    g.emit("store atomic i64 1, ptr @roop_overflow monotonic, align 8");
    g.emit(&format!("br label %{done}"));
    g.label(&done);
}
