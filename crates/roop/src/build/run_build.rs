use crate::{
    BuildArgs, CliError, Emit, build_metallib, build_ptx, clang, config_for, host_options,
    load_source, run_tool, runtime_lib,
};
use roop_llvm::{compile_all, embed_blobs};
use roop_opt::fuse_parallel;
use std::path::Path;

pub fn build(args: &BuildArgs) -> Result<(), CliError> {
    let config = config_for(&args.input)?;
    let options = host_options(&config);

    let program = load_source(&args.input, &config)?;
    roop_check::check(&program).map_err(CliError::Check)?;
    let program = fuse_parallel(&program);
    let compiled = compile_all(&program, &options).map_err(CliError::Codegen)?;

    if args.emit == Emit::Ir {
        return write_ir(args, &compiled);
    }
    let dir = std::env::temp_dir().join(format!("roop-build-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| CliError::Io("temp dir".into(), e))?;
    let metallib = compiled
        .air
        .as_deref()
        .map(|air| build_metallib(air, &dir))
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
    match args.emit {
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
    command.arg("-o").arg(&args.output);
    run_tool(command)?;
    Ok(())
}

fn write_ir(args: &BuildArgs, compiled: &roop_llvm::Compiled) -> Result<(), CliError> {
    let write = |path: &Path, text: &str| {
        std::fs::write(path, text).map_err(|e| CliError::Io(path.display().to_string(), e))
    };
    write(&args.output, &compiled.host)?;
    if let Some(air) = &compiled.air {
        write(&args.output.with_extension("air.ll"), air)?;
    }
    if let Some(ptx) = &compiled.ptx {
        write(&args.output.with_extension("ptx.ll"), ptx)?;
    }
    Ok(())
}
