use crate::{
    CodegenError, FnGen, Kind, Slot, Value, bool_type, gen_assert, kind_of, llvm_type, mem_load,
};

/// A receive fills a place that must be zero, otherwise the old value would be
/// lost and the receive could not be undone. Aggregates are not checked.
pub fn check_zero(g: &mut FnGen, slot: &Slot) -> Result<(), CodegenError> {
    let Ok(kind) = kind_of(g.ctx, &slot.ty) else {
        return Ok(());
    };
    let current = mem_load(g, slot)?;
    let ty = llvm_type(g.ctx, &slot.ty)?;
    let reg = format!("%{}", g.fresh("t"));
    let compare = match kind {
        Kind::Float => format!("fcmp oeq {ty} {}, 0.000000e+00", current.reg),
        Kind::Bool => format!("icmp eq {ty} {}, false", current.reg),
        Kind::Int | Kind::Enum => format!("icmp eq {ty} {}, 0", current.reg),
    };
    g.emit(&format!("{reg} = {compare}"));
    gen_assert(
        g,
        &Value {
            reg,
            ty: bool_type(),
        },
    )
}
