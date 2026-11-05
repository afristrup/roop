mod support;

#[test]
fn certificates_and_refinement_foundations_check() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lean");
    let lake = std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/.elan/bin/lake"))
        .filter(|lake| std::path::Path::new(lake).is_file())
        .unwrap_or_else(|| "lake".into());
    let output = std::process::Command::new(lake)
        .args(["env", "lean", "RoopLean.lean"])
        .current_dir(root)
        .output();
    let output = output.expect("Lean must be available to check the certificate");
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
