#![allow(dead_code)]

use roop_check::check;
use roop_llvm::{Compiled, Options, compile_all, compile_with};
use roop_syntax::parse;
use std::io::Write;
use std::process::{Command, Stdio};

const BREW_LLVM: &str = "/opt/homebrew/opt/llvm/bin";

fn tool(name: &str) -> Option<String> {
    [name.to_string(), format!("{BREW_LLVM}/{name}")]
        .into_iter()
        .find(|t| {
            Command::new(t)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
        })
}

pub fn ir(src: &str) -> String {
    ir_with(src, &Options::default())
}

pub fn ir_with(src: &str, options: &Options) -> String {
    let program = parse(src).unwrap();
    check(&program).unwrap();
    compile_with(&program, options).unwrap()
}

/// Runs `tool` with the IR on stdin. None when the tool is not installed.
fn run_tool(name: &str, args: &[&str], ir: &str) -> Option<std::process::Output> {
    let path = tool(name)?;
    let mut child = Command::new(path)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(ir.as_bytes())
        .unwrap();
    Some(child.wait_with_output().unwrap())
}

pub fn verify(ir: &str) {
    if let Some(out) = run_tool("opt", &["-passes=verify", "-disable-output", "-"], ir) {
        assert!(
            out.status.success(),
            "{}\n{ir}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// Exit code of `@main`, or None when `lli` is not installed. A trap shows up
/// as a missing code.
pub fn run(ir: &str) -> Option<Option<i32>> {
    run_tool("lli", &["-"], ir).map(|out| out.status.code())
}

fn runtime_lib() -> std::path::PathBuf {
    use std::sync::Once;
    static BUILD: Once = Once::new();
    BUILD.call_once(|| {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "roop-rt"])
            .status()
            .unwrap();
        assert!(status.success());
    });
    let target = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/../../target", env!("CARGO_MANIFEST_DIR")));
    std::path::PathBuf::from(target).join("debug/libroop_rt.a")
}

/// Compiles the IR modules with clang, links the roop runtime (and the Metal
/// frameworks on macOS) and returns the exit code. None when clang is missing.
pub fn run_native_modules(modules: &[&str]) -> Option<Option<i32>> {
    let named: Vec<(String, &str)> = modules
        .iter()
        .enumerate()
        .map(|(i, m)| (format!("m{i}.ll"), *m))
        .collect();
    run_native_files(&named)
}

/// Like `run_native_modules` but each source carries its file name, so C
/// harnesses can sit next to generated IR.
pub fn run_native_files(files: &[(String, &str)]) -> Option<Option<i32>> {
    let clang = tool("clang")?;
    let seed: usize = files.iter().map(|(_, m)| m.len()).sum();
    let dir = std::env::temp_dir().join(format!("roop-{}-{:x}", std::process::id(), seed));
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("prog");
    let mut command = Command::new(clang);
    for (name, text) in files {
        let src = dir.join(name);
        std::fs::write(&src, text).unwrap();
        command.arg(src);
    }
    command
        .arg(runtime_lib())
        .arg("-o")
        .arg(&exe)
        .args(["-lpthread", "-lm"]);
    if cfg!(target_os = "macos") {
        command.args(["-framework", "Metal", "-framework", "Foundation", "-lobjc"]);
    }
    let build = command.output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    Some(Command::new(&exe).status().unwrap().code())
}

pub fn run_native(ir: &str) -> Option<Option<i32>> {
    run_native_modules(&[ir])
}

pub fn compiled(src: &str, options: &Options) -> Compiled {
    let program = parse(src).unwrap();
    check(&program).unwrap();
    compile_all(&program, options).unwrap()
}

/// AIR to a `.metallib` with Apple's tools. None when they are not installed.
pub fn metallib(air_ir: &str) -> Option<Vec<u8>> {
    let dir = std::env::temp_dir().join(format!(
        "roop-air-{}-{:x}",
        std::process::id(),
        air_ir.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let (ll, air, lib) = (dir.join("k.ll"), dir.join("k.air"), dir.join("k.metallib"));
    std::fs::write(&ll, air_ir).unwrap();
    let compile = Command::new("xcrun")
        .args(["-sdk", "macosx", "metal", "-c", "-x", "ir"])
        .arg(&ll)
        .arg("-o")
        .arg(&air)
        .output()
        .ok()?;
    assert!(
        compile.status.success(),
        "{}\n{air_ir}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let link = Command::new("xcrun")
        .args(["-sdk", "macosx", "metallib"])
        .arg(&air)
        .arg("-o")
        .arg(&lib)
        .output()
        .ok()?;
    assert!(
        link.status.success(),
        "{}",
        String::from_utf8_lossy(&link.stderr)
    );
    std::fs::read(&lib).ok()
}

/// IR module defining the symbols the host code declares as external.
pub fn metallib_blob_module(bytes: &[u8]) -> String {
    let escaped: String = bytes.iter().map(|b| format!("\\{b:02X}")).collect();
    format!(
        "@roop_metallib = constant [{n} x i8] c\"{escaped}\"\n@roop_metallib_len = constant i64 {n}\n",
        n = bytes.len()
    )
}
