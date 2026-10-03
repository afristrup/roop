use crate::{
    CodegenError, Ctx, Dialect, Dir, FnGen, GenOutput, Slot, function_attrs, gen_block,
    gen_unwinding, llvm_type, mem_store,
};
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
    unwinding: bool,
) -> Result<GenOutput, CodegenError> {
    let mut g = FnGen::new(ctx, symbol.into(), Dialect::Host);
    let mut signature = Vec::new();
    if let Some(name) = self_struct {
        signature.push("ptr %arg_self".to_string());
        g.vars.push((
            "self".into(),
            Slot {
                addr: "%arg_self".into(),
                ty: Type::Named(name.into()),
                space: 0,
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
                space: 0,
            },
            ty => {
                let addr = g.alloca(&llty);
                let slot = Slot {
                    addr,
                    ty: ty.clone(),
                    space: 0,
                };
                mem_store(&mut g, &slot, &arg)?;
                slot
            }
        };
        g.vars.push((param.name.clone(), slot));
    }
    let (ret, tail) = if unwinding {
        let fail = g.fresh("L");
        gen_unwinding(&mut g, body, dir, &fail)?;
        g.emit("ret i32 0");
        g.label(&fail);
        g.emit("ret i32 1");
        ("i32", "")
    } else {
        gen_block(&mut g, body, dir)?;
        ("void", "  ret void\n")
    };
    let text = format!(
        "define {ret} @{symbol}({}){} {{\nentry:\n{}{}{tail}}}\n{}{}",
        signature.join(", "),
        function_attrs(ctx),
        g.allocas,
        g.body,
        g.outlined.join("\n"),
        g.globals.join("\n")
    );
    Ok(GenOutput {
        text,
        kernels: g.kernels,
    })
}
