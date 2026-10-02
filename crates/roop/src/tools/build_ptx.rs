use crate::{CliError, find_tool, run_tool};
use std::process::Command;

/// NVPTX IR to PTX text with `llc`.
pub fn build_ptx(ptx_ir: &str, arch: &str) -> Result<String, CliError> {
    let llc = find_tool("llc").ok_or_else(|| CliError::Tool("llc not found".into()))?;
    let dir = std::env::temp_dir().join(format!("roop-ptx-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| CliError::Io("temp dir".into(), e))?;
    let input = dir.join("kernels.ll");
    std::fs::write(&input, ptx_ir).map_err(|e| CliError::Io("writing PTX IR".into(), e))?;
    let mut command = Command::new(llc);
    command
        .args([
            "-mtriple=nvptx64-nvidia-cuda",
            &format!("-mcpu={arch}"),
            "-o",
            "-",
        ])
        .arg(&input);
    let out = run_tool(command)?;
    String::from_utf8(out).map_err(|_| CliError::Tool("llc produced invalid PTX".into()))
}
