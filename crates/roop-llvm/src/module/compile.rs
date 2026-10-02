use crate::{
    CodegenError, Compiled, Ctx, Dialect, Dir, Kernel, Options, air_module, gen_function,
    module_header, ptx_module, try_variants, type_decls,
};
use roop_syntax::{Item, Program};

/// Compiles a checked program to textual LLVM IR for the host, host only.
pub fn compile(program: &Program) -> Result<String, CodegenError> {
    compile_with(program, &Options::default())
}

pub fn compile_with(program: &Program, options: &Options) -> Result<String, CodegenError> {
    Ok(compile_all(program, options)?.host)
}

/// Each `fn f` becomes `@f` and its inverse `@f_inv`; struct constructors
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
         declare ptr @roop_chan_new(i64)\n\
         declare void @roop_chan_send(ptr, ptr)\n\
         declare i32 @roop_chan_recv(ptr, ptr, ptr)\n\
         declare i64 @roop_chan_len(ptr)\n\
         declare void @roop_chan_free(ptr)\n\
         declare ptr @roop_chan_snapshot(ptr)\n\
         declare void @roop_chan_restore(ptr, ptr)\n\
         declare void @roop_chan_snapshot_free(ptr)\n\
         declare void @roop_concurrent(i64, ptr, ptr)\n\
         declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)\n\
         @roop_metallib = external constant i8\n\
         @roop_metallib_len = external constant i64\n\
         @roop_ptx = external constant i8\n\n",
    );
    let variants = try_variants(&ctx);
    let mut kernels: Vec<Kernel> = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                let has_inverse = !ctx.irreversible.contains(f.name.as_str());
                for (symbol, dir) in [
                    (f.name.clone(), Dir::Forward),
                    (format!("{}_inv", f.name), Dir::Backward),
                ]
                .into_iter()
                .filter(|(_, dir)| has_inverse || *dir == Dir::Forward)
                {
                    let out = gen_function(&ctx, &symbol, None, &f.params, &f.body, dir, false)?;
                    host.push_str(&out.text);
                    host.push('\n');
                    kernels.extend(out.kernels);
                }
                for inverse in [false, true] {
                    if !variants.contains(&(f.name.clone(), inverse)) {
                        continue;
                    }
                    let (dir, suffix) = if inverse {
                        (Dir::Backward, "_inv_try")
                    } else {
                        (Dir::Forward, "_try")
                    };
                    let symbol = format!("{}{suffix}", f.name);
                    let out = gen_function(&ctx, &symbol, None, &f.params, &f.body, dir, true)?;
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
                    let out = gen_function(
                        &ctx,
                        &symbol,
                        Some(&def.name),
                        params,
                        body,
                        Dir::Forward,
                        false,
                    )?;
                    host.push_str(&out.text);
                    host.push('\n');
                    kernels.extend(out.kernels);
                }
            }
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) | Item::Session(_) => {}
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
