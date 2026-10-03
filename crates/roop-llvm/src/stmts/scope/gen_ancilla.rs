use crate::{Clear, CodegenError, Dir, FnGen, declare_ancilla, gen_block};
use roop_check::uncompute_pairs;
use roop_syntax::{Block, Expr, Stmt, Type};

pub fn gen_ancilla(
    g: &mut FnGen,
    name: &str,
    ty: &Type,
    init: &Expr,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    let slot = declare_ancilla(g, ty, init)?;
    g.vars.push((name.into(), slot.clone()));
    let enclosing = g.clears.len();
    let ctx = g.ctx;
    if ctx.options.clear_ancillas && matches!(ty, Type::Array(..)) {
        for (first, second) in uncompute_pairs(name, init, body, &ctx.mutability, &ctx.effectful) {
            let last: &Stmt = match dir {
                Dir::Forward => second,
                Dir::Backward => first,
            };
            g.clears.push(Clear {
                stmt: std::ptr::from_ref(last) as usize,
                dir,
                slot: slot.clone(),
            });
        }
    }
    let result = gen_block(g, body, dir);
    g.clears.truncate(enclosing);
    g.vars.pop();
    result
}
