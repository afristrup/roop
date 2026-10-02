use crate::{CliError, find_tool, run_tool};
use std::io::Write;
use std::process::{Command, Stdio};

/// NVPTX IR to PTX text with `llc`.
pub fn build_ptx(ptx_ir: &str, arch: &str) -> Result<String, CliError> {
    let llc = find_tool("llc").ok_or_else(|| CliError::Tool("llc not found".into()))?;
    let mut llc = Command::new(llc);
    llc.args([
        "-mtriple=nvptx64-nvidia-cuda",
        &format!("-mcpu={arch}"),
        "-o",
        "-",
    ]);
    llc.stdin(Stdio::piped());
    let dir = std::env::temp_dir().join(format!("roop-ptx-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| CliError::Io("temp dir".into(), e))?;
    let input = dir.join("kernels.ll");
    let mut file =
        std::fs::File::create(&input).map_err(|e| CliError::Io("writing PTX IR".into(), e))?;
    file.write_all(ptx_ir.as_bytes())
        .map_err(|e| CliError::Io("writing PTX IR".into(), e))?;
    llc.arg(&input).stdin(Stdio::null());
    let out = run_tool(llc)?;
    String::from_utf8(out).map_err(|_| CliError::Tool("llc produced invalid PTX".into()))
}
