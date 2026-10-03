use crate::{CodegenError, FnGen, chan_handle, gen_place, llvm_type, same_type};
use roop_syntax::Place;

/// Sends the value at `source` and leaves zero behind.
pub fn move_out(g: &mut FnGen, chan: &str, source: &Place) -> Result<(), CodegenError> {
    let slot = gen_place(g, source)?;
    let (handle, ty) = chan_handle(g, chan)?;
    same_type(&ty, &slot.ty)?;
    g.emit(&format!(
        "call void @roop_chan_send(ptr {handle}, ptr {})",
        slot.addr
    ));
    let llty = llvm_type(g.ctx, &slot.ty)?;
    g.emit(&format!("store {llty} zeroinitializer, ptr {}", slot.addr));
    Ok(())
}
