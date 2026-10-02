#![allow(dead_code)]

use roop_check::check;
use roop_llvm::{Options, compile_with};
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

/// Compiles the IR with clang, links the roop runtime and returns the exit
/// code. None when clang is not installed.
pub fn run_native(ir: &str) -> Option<Option<i32>> {
    let clang = tool("clang")?;
    let dir = std::env::temp_dir().join(format!("roop-{}-{:x}", std::process::id(), ir.len()));
    std::fs::create_dir_all(&dir).unwrap();
    let (src, exe) = (dir.join("prog.ll"), dir.join("prog"));
    std::fs::write(&src, ir).unwrap();
    let build = Command::new(clang)
        .arg(&src)
        .arg(runtime_lib())
        .args(["-o"])
        .arg(&exe)
        .args(["-lpthread", "-lm"])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    Some(Command::new(&exe).status().unwrap().code())
}
