use crate::{
    CodegenError, Compiled, Ctx, Dialect, Dir, Kernel, Options, air_module, entry_symbol,
    gen_function, module_header, ptx_module, try_variants, type_decls,
};
use roop_syntax::{Item, Program};

/// The C `main`: records the arguments for the runtime, runs `roop_main`, and
/// returns the status it leaves.
fn main_shim(params: usize, declare_commit: bool, history_limit: Option<u64>) -> String {
    let (arg, passed) = if params == 0 {
        ("", "")
    } else {
        (
            "  %status = alloca i64\n  store i64 0, ptr %status\n",
            "ptr %status",
        )
    };
    let ret = if params == 0 {
        "0".to_string()
    } else {
        "%code".to_string()
    };
    let load = if params == 0 {
        String::new()
    } else {
        "  %s = load i64, ptr %status\n  %code = trunc i64 %s to i32\n".to_string()
    };
    let declare = if declare_commit {
        "declare void @roop_commit()\n\n"
    } else {
        ""
    };
    let (declare_limit, set_limit) = match history_limit {
        Some(limit) => (
            "declare void @roop_set_history_limit(i64)\n\n".to_string(),
            format!("  call void @roop_set_history_limit(i64 {limit})\n"),
        ),
        None => (String::new(), String::new()),
    };
    format!(
        "{declare}{declare_limit}define i32 @main(i32 %argc, ptr %argv) {{\nentry:\n  call void @roop_set_args(i32 %argc, ptr %argv)\n{set_limit}{arg}  call void @roop_main({passed})\n{load}  call void @roop_commit()\n  ret i32 {ret}\n}}\n\n"
    )
}

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
         declare i64 @llvm.fptosi.sat.i64.f64(double)\n\
         declare i8 @llvm.fptoui.sat.i8.f64(double)\n\
         declare void @roop_set_args(i32, ptr)\n\
         declare void @roop_keep(ptr, i64)\n\
         declare void @roop_unkeep(ptr, i64)\n\
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
            Item::Fn(f) if f.external => {
                let pointers = vec!["ptr"; f.params.len()].join(", ");
                host.push_str(&format!("declare void @{}({pointers})\n", f.name));
                if f.world {
                    host.push_str(&format!("declare void @{}_inv({pointers})\n", f.name));
                }
                host.push('\n');
            }
            Item::Fn(f) => {
                let has_inverse = !ctx.irreversible.contains(f.name.as_str());
                for (symbol, dir) in [
                    (entry_symbol(&f.name), Dir::Forward),
                    (format!("{}_inv", entry_symbol(&f.name)), Dir::Backward),
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
                    let symbol = format!("{}{suffix}", entry_symbol(&f.name));
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
    if !options.no_entry
        && let Some(main) = program.items.iter().find_map(|item| match item {
            Item::Fn(f) if f.name == "main" && !f.external => Some(f),
            _ => None,
        })
    {
        let declared = program
            .items
            .iter()
            .any(|item| matches!(item, Item::Fn(f) if f.external && f.name == "roop_commit"));
        host.push_str(&main_shim(
            main.params.len(),
            !declared,
            options.history_limit,
        ));
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
