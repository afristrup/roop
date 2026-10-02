#![allow(dead_code)]

use roop_check::check;
use roop_llvm::compile;
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
    let program = parse(src).unwrap();
    check(&program).unwrap();
    compile(&program).unwrap()
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
