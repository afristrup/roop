use crate::FnGen;

/// Emits `%t = instr i64 a, b` and returns `%t`.
pub fn int_op(g: &mut FnGen, instr: &str, a: &str, b: &str) -> String {
    let reg = format!("%{}", g.fresh("t"));
    g.emit(&format!("{reg} = {instr} i64 {a}, {b}"));
    reg
}
