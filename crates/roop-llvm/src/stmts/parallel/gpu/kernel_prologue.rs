use crate::{Dialect, FnGen, params_space, ptr_type};

/// Loads `lo`, `step` and `count` from the params buffer and returns the
/// 64-bit global thread index.
pub fn kernel_prologue(k: &mut FnGen) -> (String, String, String, String) {
    let i64p = ptr_type(k.dialect, "i64", params_space(k.dialect));
    let mut loaded = Vec::new();
    for index in 0..3 {
        let p = format!("%{}", k.fresh("t"));
        k.emit(&format!(
            "{p} = getelementptr inbounds i64, {i64p} %params, i64 {index}"
        ));
        let v = format!("%{}", k.fresh("t"));
        k.emit(&format!("{v} = load i64, {i64p} {p}"));
        loaded.push(v);
    }
    let tid = match k.dialect {
        Dialect::Nvptx => {
            let (t, b, n) = ("%tx", "%bx", "%nx");
            k.emit(&format!("{t} = call i32 @llvm.nvvm.read.ptx.sreg.tid.x()"));
            k.emit(&format!(
                "{b} = call i32 @llvm.nvvm.read.ptx.sreg.ctaid.x()"
            ));
            k.emit(&format!("{n} = call i32 @llvm.nvvm.read.ptx.sreg.ntid.x()"));
            k.emit("%gx = mul i32 %bx, %nx");
            k.emit("%tid = add i32 %gx, %tx");
            "%tid"
        }
        _ => "%tid",
    };
    let wide = format!("%{}", k.fresh("t"));
    k.emit(&format!("{wide} = zext i32 {tid} to i64"));
    (
        loaded[0].clone(),
        loaded[1].clone(),
        loaded[2].clone(),
        wide,
    )
}
