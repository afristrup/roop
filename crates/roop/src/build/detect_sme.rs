use std::process::Command;

/// Whether this machine has the SME matrix unit with double precision, which
/// only the M4 and later of Apple Silicon do.
pub fn detect_sme() -> bool {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return false;
    }
    Command::new("sysctl")
        .args(["-n", "hw.optional.arm.FEAT_SME_F64F64"])
        .output()
        .is_ok_and(|out| String::from_utf8_lossy(&out.stdout).trim() == "1")
}
