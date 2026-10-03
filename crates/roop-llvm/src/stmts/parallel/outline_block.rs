use crate::{AbortMode, CodegenError, Dir, FnGen, Slot, function_attrs, gen_block};
use roop_syntax::{Block, Type};

/// Moves a block into its own function taking `(env, iteration)`. The
/// environment holds the address of every variable visible at the call site.
/// A loop body also gets its induction variable as a private local. A body
/// launched from inside a `try` reports failures through the shared abort flag
/// instead of stopping the program.
pub fn outline_block(
    g: &mut FnGen,
    var: Option<&str>,
    body: &Block,
    dir: Dir,
    tag: &str,
) -> Result<String, CodegenError> {
    let id = g.fresh("");
    let symbol = format!("{}.{tag}{id}", g.symbol);
    let mut child = FnGen::new(g.ctx, symbol.clone(), g.dialect);
    child.abort = if g.abort == AbortMode::Trap {
        AbortMode::Trap
    } else {
        AbortMode::Flag
    };
    child.chan_types = g.chan_types.clone();
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
    if let Some(var) = var {
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
    }
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
