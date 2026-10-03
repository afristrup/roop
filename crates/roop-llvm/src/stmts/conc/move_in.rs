use crate::{
    AbortMode, CodegenError, FnGen, Value, abort_slot, bool_type, chan_handle, check_zero,
    gen_assert, gen_place, same_type,
};
use roop_syntax::Place;

/// Receives the next message into `target`, which must be zero. A wait that
/// ends because a `try` aborted counts as a failed assertion.
pub fn move_in(g: &mut FnGen, chan: &str, target: &Place) -> Result<(), CodegenError> {
    let slot = gen_place(g, target)?;
    let (handle, ty) = chan_handle(g, chan)?;
    same_type(&ty, &slot.ty)?;
    check_zero(g, &slot)?;
    let abort = match g.abort {
        AbortMode::Trap => "null".to_string(),
        _ => abort_slot(g)?,
    };
    let status = format!("%{}", g.fresh("t"));
    g.emit(&format!(
        "{status} = call i32 @roop_chan_recv(ptr {handle}, ptr {}, ptr {abort})",
        slot.addr
    ));
    let ok = format!("%{}", g.fresh("t"));
    g.emit(&format!("{ok} = icmp eq i32 {status}, 0"));
    gen_assert(
        g,
        &Value {
            reg: ok,
            ty: bool_type(),
        },
    )
}
