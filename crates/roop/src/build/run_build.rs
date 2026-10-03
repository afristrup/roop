use crate::{
    BuildArgs, BuildDir, CliError, Emit, build_metallib, build_ptx, clang, config_for,
    host_options, load_source, load_with_tests, run_tool, runtime_lib,
};
use roop_llvm::{compile_all, embed_blobs};
use roop_opt::fuse_parallel;
use std::path::Path;

pub fn build(args: &BuildArgs) -> Result<(), CliError> {
    let config = config_for(&args.input)?;
    let program = if args.keep_tests {
        load_with_tests(&args.input, &config)?
    } else {
        load_source(&args.input, &config)?
    };
    build_program(args, &config, &program)
}

/// Checks and compiles a program that is already loaded.
pub fn build_program(
    args: &BuildArgs,
    config: &roop_config::Config,
    program: &roop_syntax::Program,
) -> Result<(), CliError> {
    let mut options = host_options(config);
    options.no_entry = !args.link.is_empty();
    let has_main = program
        .items
        .iter()
        .any(|item| matches!(item, roop_syntax::Item::Fn(f) if f.name == "main" && !f.external));
    let emit = args.emit.unwrap_or(if has_main || !args.link.is_empty() {
        Emit::Executable
    } else {
        Emit::Object
    });
    let output = args.output.clone().unwrap_or_else(|| {
        args.input.with_extension(match emit {
            Emit::Ir => "ll",
            Emit::Object => "o",
            Emit::Executable => "",
        })
    });
    roop_check::check(program).map_err(CliError::Check)?;
    let program = fuse_parallel(program);
    let compiled = compile_all(&program, &options).map_err(CliError::Codegen)?;

    if emit == Emit::Ir {
        return write_ir(&output, &compiled);
    }
    let scratch = BuildDir::create()?;
    let dir = scratch.path();
    let metallib = compiled
        .air
        .as_deref()
        .map(|air| build_metallib(air, dir))
        .transpose()?;
    let ptx = compiled
        .ptx
        .as_deref()
        .map(|ir| build_ptx(ir, "sm_80"))
        .transpose()?;
    let host = embed_blobs(&compiled.host, metallib.as_deref(), ptx.as_deref());
    let host_path = dir.join("program.ll");
    std::fs::write(&host_path, host).map_err(|e| CliError::Io("writing IR".into(), e))?;

    let mut command = clang()?;
    command.arg("-O2").arg(&host_path);
    match emit {
        Emit::Object => {
            command.arg("-c");
        }
        _ => {
            command
                .args(&args.link)
                .arg(runtime_lib())
                .args(["-lpthread", "-lm"]);
            if cfg!(target_os = "macos") {
                command.args(["-framework", "Metal", "-framework", "Foundation", "-lobjc"]);
            }
        }
    }
    command.arg("-o").arg(&output);
    run_tool(command)?;
    Ok(())
}

fn write_ir(output: &Path, compiled: &roop_llvm::Compiled) -> Result<(), CliError> {
    let write = |path: &Path, text: &str| {
        std::fs::write(path, text).map_err(|e| CliError::Io(path.display().to_string(), e))
    };
    write(output, &compiled.host)?;
    if let Some(air) = &compiled.air {
        write(&output.with_extension("air.ll"), air)?;
    }
    if let Some(ptx) = &compiled.ptx {
        write(&output.with_extension("ptx.ll"), ptx)?;
    }
    Ok(())
}
