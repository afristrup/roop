use crate::{CodegenError, Dir, FnGen, Slot, function_attrs, gen_block};
use roop_syntax::{Block, Type};

/// Moves a loop body into its own function taking `(env, iteration)`. The
/// environment holds the address of every variable visible at the loop.
pub fn outline_body(
    g: &mut FnGen,
    var: &str,
    body: &Block,
    dir: Dir,
) -> Result<String, CodegenError> {
    let id = g.fresh("");
    let symbol = format!("{}.par{id}", g.symbol);
    let mut child = FnGen::new(g.ctx, symbol.clone(), g.dialect);
    let n = g.vars.len();
    for (i, (name, slot)) in g.vars.clone().into_iter().enumerate() {
        let at = format!("%{}", child.fresh("t"));
        child.emit(&format!(
            "{at} = getelementptr inbounds [{n} x ptr], ptr %env, i64 0, i64 {i}"
        ));
        let addr = format!("%{}", child.fresh("t"));
        child.emit(&format!("{addr} = load ptr, ptr {at}"));
        child.vars.push((
            name,
            Slot {
                addr,
                ty: slot.ty,
                space: slot.space,
            },
        ));
    }
    let iv = child.alloca("i64");
    child.emit(&format!("store i64 %iter, ptr {iv}"));
    child.vars.push((
        var.into(),
        Slot {
            addr: iv,
            ty: Type::Named("i64".into()),
            space: 0,
        },
    ));
    gen_block(&mut child, body, dir)?;
    g.outlined.push(format!(
        "define internal void @{symbol}(ptr %env, i64 %iter){} {{\nentry:\n{}{}  ret void\n}}\n",
        function_attrs(g.ctx),
        child.allocas,
        child.body
    ));
    g.outlined.append(&mut child.outlined);
    Ok(symbol)
}
