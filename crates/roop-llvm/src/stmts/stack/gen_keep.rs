use crate::{CodegenError, Dir, FnGen, gen_place, layout};
use roop_syntax::Place;

/// `keep x` hands the bytes of `x` to the world, which remembers them, and
/// leaves zero; backward the world gives the last ones back.
pub fn gen_keep(g: &mut FnGen, place: &Place, dir: Dir) -> Result<(), CodegenError> {
    let slot = gen_place(g, place)?;
    let (size, _) = layout(g.ctx, &slot.ty)?;
    let symbol = match dir {
        Dir::Forward => "roop_keep",
        Dir::Backward => "roop_unkeep",
    };
    g.emit(&format!(
        "call void @{symbol}(ptr {}, i64 {size})",
        slot.addr
    ));
    Ok(())
}
