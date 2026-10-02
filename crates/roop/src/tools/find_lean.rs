use std::process::{Command, Stdio};

/// `lean` on `PATH`, else the elan install.
pub fn find_lean() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    ["lean".to_string(), format!("{home}/.elan/bin/lean")]
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
