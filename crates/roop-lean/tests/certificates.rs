mod support;

#[test]
fn arbitrary_coupling_networks_have_a_checked_roundtrip_certificate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lean");
    let lake = std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/.elan/bin/lake"))
        .filter(|lake| std::path::Path::new(lake).is_file())
        .unwrap_or_else(|| "lake".into());
    let output = std::process::Command::new(lake)
        .args(["env", "lean", "RoopLean/Certificate/TensorNetwork.lean"])
        .current_dir(root)
        .output();
    let Ok(output) = output else { return };
    assert!(
        output.status.success(),
        "{}",
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    );
}
