mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{compile, project, roop, text};

fn python_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("python")
}

#[test]
fn what_the_exporter_writes_compiles_and_agrees_with_the_reference() {
    let dir = project("exporter");
    let exported = dir.join("exported.json");
    let run = Command::new("python3")
        .args(["-m", "unittest", "-q"])
        .current_dir(python_dir())
        .env("WEAVE_EXPORT_OUT", &exported)
        .output();
    let Ok(run) = run else {
        eprintln!("python3 is not installed, so the exporter is not tested");
        return;
    };
    assert!(run.status.success(), "{}", text(&run));
    let model = serde_json::from_str(&std::fs::read_to_string(&exported).unwrap()).unwrap();
    compile(&dir, &model, &["--tests"]);
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("1 passed, 0 failed"), "{}", text(&out));
}
