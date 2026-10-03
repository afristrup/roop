use std::process::{Command, Stdio};

const BREW_LLVM: &str = "/opt/homebrew/opt/llvm/bin";

/// A usable path for an LLVM tool: on `PATH`, else Homebrew's keg.
pub fn find_tool(name: &str) -> Option<String> {
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
