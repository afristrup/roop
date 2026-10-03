use crate::{CodegenError, Dir, FnGen, declare_ancilla, gen_block};
use roop_syntax::{Block, Expr, Type};

pub fn gen_ancilla(
    g: &mut FnGen,
    name: &str,
    ty: &Type,
    init: &Expr,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    let slot = declare_ancilla(g, ty, init)?;
    g.vars.push((name.into(), slot));
    let result = gen_block(g, body, dir);
    g.vars.pop();
    result
}
