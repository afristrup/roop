use std::process::Command;

/// The LLVM CPU model of this machine, e.g. `apple-m4`. Only Apple Silicon
/// is recognised; elsewhere the generic target is used.
pub fn detect_cpu() -> Option<String> {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return None;
    }
    let out = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()?;
    let brand = String::from_utf8(out.stdout).ok()?.trim().to_lowercase();
    let model = brand.strip_prefix("apple ")?;
    Some(format!("apple-{model}"))
}
