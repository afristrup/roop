mod support;

use roop_weave::{emit_certificate, emit_model, parse_model, source_fingerprint};
use std::process::Command;
use support::{attention, leapfrog, mlp, model, project, roop, text};

#[test]
fn a_generated_certificate_checks_an_arbitrary_coupling_plan() {
    let model = parse_model(
        &model(
            "net",
            4,
            1,
            vec![
                leapfrog("tanh", 3, 4, 1),
                mlp("relu", 2, 4, 2),
                attention(2, 2, 3),
            ],
        )
        .to_string(),
    )
    .unwrap();
    let source = emit_model(&model, false, None, false);
    let text = emit_certificate(&model, &source);
    assert!(
        text.contains(&format!("{:016x}", source_fingerprint(&source))),
        "{text}"
    );
    assert!(text.contains("net_update_4"), "{text}");
    assert!(text.contains("intoFirst := true"), "{text}");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../roop-lean/lean");
    let file = std::env::temp_dir().join(format!(
        "roop-weave-certificate-{}.lean",
        std::process::id()
    ));
    std::fs::write(&file, text).unwrap();
    let lake = std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/.elan/bin/lake"))
        .filter(|lake| std::path::Path::new(lake).is_file())
        .unwrap_or_else(|| "lake".into());
    let output = Command::new(lake)
        .args(["env", "lean", file.to_str().unwrap()])
        .current_dir(root)
        .output();
    let _ = std::fs::remove_file(&file);
    let output = output.expect("Lean must be available to check the certificate");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn the_cli_writes_a_checked_certificate() {
    let dir = project("certificate-cli");
    let specification = model("cli_net", 4, 1, vec![mlp("relu", 2, 4, 1)]);
    std::fs::write(dir.join("model.json"), specification.to_string()).unwrap();
    let output = roop(
        &dir,
        &["weave", "model.json", "--certificate", "model.lean"],
    );
    assert!(output.status.success(), "{}", text(&output));
    let source = String::from_utf8(output.stdout).unwrap();
    let certificate = std::fs::read_to_string(dir.join("model.lean")).unwrap();
    assert!(
        certificate.contains(&format!("{:016x}", source_fingerprint(&source))),
        "{certificate}"
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../roop-lean/lean");
    let lake = std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/.elan/bin/lake"))
        .filter(|lake| std::path::Path::new(lake).is_file())
        .unwrap_or_else(|| "lake".into());
    let output = Command::new(lake)
        .args(["env", "lean", dir.join("model.lean").to_str().unwrap()])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_deep_certificate_checks_without_expanding_updates() {
    let layers = (0..1024).map(|seed| mlp("relu", 2, 64, seed + 1)).collect();
    let model = parse_model(&model("deep", 64, 1, layers).to_string()).unwrap();
    let source = emit_model(&model, false, None, false);
    let text = emit_certificate(&model, &source);
    assert_eq!(text.matches("opaque deep_update_").count(), 1024);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../roop-lean/lean");
    let file = std::env::temp_dir().join(format!(
        "roop-weave-deep-certificate-{}.lean",
        std::process::id()
    ));
    std::fs::write(&file, text).unwrap();
    let lake = std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/.elan/bin/lake"))
        .filter(|lake| std::path::Path::new(lake).is_file())
        .unwrap_or_else(|| "lake".into());
    let output = Command::new(lake)
        .args(["env", "lean", file.to_str().unwrap()])
        .current_dir(root)
        .output();
    let _ = std::fs::remove_file(&file);
    let output = output.expect("Lean must be available to check the certificate");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
