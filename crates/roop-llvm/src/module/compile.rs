use crate::{CodegenError, Ctx, Dir, gen_function, type_decls};
use roop_syntax::{Item, Program};

/// Compiles a checked program to textual LLVM IR. Each `rev fn f` becomes
/// `@f` and its inverse `@f_inv`; struct constructors become
/// `@S_build` and `@S_unbuild`.
pub fn compile(program: &Program) -> Result<String, CodegenError> {
    let ctx = Ctx::new(program);
    let mut out = type_decls(&ctx, program)?;
    out.push_str("\ndeclare void @llvm.trap() noreturn nounwind\ndeclare void @roop_parallel_for(i64, i64, i64, ptr, ptr)\n\n");
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                for (symbol, dir) in [
                    (f.name.clone(), Dir::Forward),
                    (format!("{}_inv", f.name), Dir::Backward),
                ] {
                    out.push_str(&gen_function(&ctx, &symbol, None, &f.params, &f.body, dir)?);
                    out.push('\n');
                }
            }
            Item::Struct(def) => {
                for (suffix, ctor) in [("build", &def.build), ("unbuild", &def.unbuild)] {
                    let Some(ctor) = ctor else { continue };
                    let symbol = format!("{}_{suffix}", def.name);
                    let ir = gen_function(
                        &ctx,
                        &symbol,
                        Some(&def.name),
                        &ctor.params,
                        &ctor.body,
                        Dir::Forward,
                    )?;
                    out.push_str(&ir);
                    out.push('\n');
                }
            }
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) => {}
        }
    }
    Ok(out)
}
