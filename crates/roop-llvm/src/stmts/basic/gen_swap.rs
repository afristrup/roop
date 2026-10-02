use crate::{CodegenError, FnGen, gen_place, mem_load, mem_store, same_type};
use roop_syntax::Place;

pub fn gen_swap(g: &mut FnGen, a: &Place, b: &Place) -> Result<(), CodegenError> {
    let (a, b) = (gen_place(g, a)?, gen_place(g, b)?);
    same_type(&a.ty, &b.ty)?;
    let (x, y) = (mem_load(g, &a)?, mem_load(g, &b)?);
    mem_store(g, &a, &y.reg)?;
    mem_store(g, &b, &x.reg)
}
