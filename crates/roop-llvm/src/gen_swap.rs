use super::{CodegenError, FnGen, gen_place, llvm_type, same_type};
use roop_syntax::Place;

pub fn gen_swap(g: &mut FnGen, a: &Place, b: &Place) -> Result<(), CodegenError> {
    let (a, b) = (gen_place(g, a)?, gen_place(g, b)?);
    same_type(&a.ty, &b.ty)?;
    let ty = llvm_type(g.ctx, &a.ty)?;
    let (x, y) = (format!("%{}", g.fresh("t")), format!("%{}", g.fresh("t")));
    g.emit(&format!("{x} = load {ty}, ptr {}", a.addr));
    g.emit(&format!("{y} = load {ty}, ptr {}", b.addr));
    g.emit(&format!("store {ty} {y}, ptr {}", a.addr));
    g.emit(&format!("store {ty} {x}, ptr {}", b.addr));
    Ok(())
}
