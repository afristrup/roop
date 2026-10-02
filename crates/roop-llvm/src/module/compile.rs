use crate::{
    CodegenError, Compiled, Ctx, Dialect, Dir, Kernel, Options, air_module, gen_function,
    module_header, ptx_module, type_decls,
};
use roop_syntax::{Item, Program};

/// Compiles a checked program to textual LLVM IR for the host, host only.
pub fn compile(program: &Program) -> Result<String, CodegenError> {
    compile_with(program, &Options::default())
}

pub fn compile_with(program: &Program, options: &Options) -> Result<String, CodegenError> {
    Ok(compile_all(program, options)?.host)
}

/// Each `rev fn f` becomes `@f` and its inverse `@f_inv`; struct constructors
/// become `@S_build` and `@S_unbuild`. GPU loops add device modules.
pub fn compile_all(program: &Program, options: &Options) -> Result<Compiled, CodegenError> {
    let ctx = Ctx::new(program, options);
    let types = type_decls(&ctx, program)?;
    let mut host = module_header(options);
    host.push_str(&types);
    host.push_str(
        "%RoopBuf = type { ptr, i64, i32 }\n\n\
         declare void @llvm.trap() noreturn nounwind\n\
         declare void @roop_parallel_for(i64, i64, i64, ptr, ptr)\n\
         declare i32 @roop_gpu_dispatch(i32, ptr, i64, ptr, ptr, i64, i64, i64, i64)\n\
         @roop_metallib = external constant i8\n\
         @roop_metallib_len = external constant i64\n\
         @roop_ptx = external constant i8\n\n",
    );
    let mut kernels: Vec<Kernel> = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                for (symbol, dir) in [
                    (f.name.clone(), Dir::Forward),
                    (format!("{}_inv", f.name), Dir::Backward),
                ] {
                    let out = gen_function(&ctx, &symbol, None, &f.params, &f.body, dir)?;
                    host.push_str(&out.text);
                    host.push('\n');
                    kernels.extend(out.kernels);
                }
            }
            Item::Struct(def) => {
                for (suffix, ctor) in [("build", &def.build), ("unbuild", &def.unbuild)] {
                    let Some(ctor) = ctor else { continue };
                    let symbol = format!("{}_{suffix}", def.name);
                    let (params, body) = (&ctor.params, &ctor.body);
                    let out =
                        gen_function(&ctx, &symbol, Some(&def.name), params, body, Dir::Forward)?;
                    host.push_str(&out.text);
                    host.push('\n');
                    kernels.extend(out.kernels);
                }
            }
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) => {}
        }
    }
    let pick = |dialect: Dialect| -> Vec<&Kernel> {
        kernels.iter().filter(|k| k.dialect == dialect).collect()
    };
    let (air, ptx) = (pick(Dialect::Air), pick(Dialect::Nvptx));
    Ok(Compiled {
        air: (!air.is_empty()).then(|| air_module(&types, &air)),
        ptx: (!ptx.is_empty()).then(|| ptx_module(&types, &ptx)),
        host,
    })
}
