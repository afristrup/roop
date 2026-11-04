use std::process::Command;

#[test]
fn cslib_execution_bridge_builds() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lean");
    let Ok(output) = Command::new("lake").arg("build").current_dir(dir).output() else {
        return;
    };
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
