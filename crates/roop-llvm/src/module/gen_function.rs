use crate::{CodegenError, Ctx, Dir, FnGen, Slot, gen_block, llvm_type};
use roop_syntax::{Block, Param, Type};

/// Emits one function. `self_struct` adds a leading `self` pointer, used for
/// struct constructors.
pub fn gen_function(
    ctx: &Ctx,
    symbol: &str,
    self_struct: Option<&str>,
    params: &[Param],
    body: &Block,
    dir: Dir,
) -> Result<String, CodegenError> {
    let mut g = FnGen::new(ctx);
    let mut signature = Vec::new();
    if let Some(name) = self_struct {
        signature.push("ptr %arg_self".to_string());
        g.vars.push((
            "self".into(),
            Slot {
                addr: "%arg_self".into(),
                ty: Type::Named(name.into()),
            },
        ));
    }
    for param in params {
        let arg = format!("%arg_{}", param.name);
        let llty = llvm_type(ctx, &param.ty)?;
        signature.push(format!("{llty} {arg}"));
        let slot = match &param.ty {
            Type::Ref { inner, .. } => Slot {
                addr: arg,
                ty: (**inner).clone(),
            },
            ty => {
                let addr = g.alloca(&llty);
                g.emit(&format!("store {llty} {arg}, ptr {addr}"));
                Slot {
                    addr,
                    ty: ty.clone(),
                }
            }
        };
        g.vars.push((param.name.clone(), slot));
    }
    gen_block(&mut g, body, dir)?;
    Ok(format!(
        "define void @{symbol}({}) {{\nentry:\n{}{}  ret void\n}}\n",
        signature.join(", "),
        g.allocas,
        g.body
    ))
}
