#![allow(dead_code)]

use roop_lean::{Translation, translate};
use roop_syntax::parse;
use std::process::Command;

fn lean() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    ["lean".to_string(), format!("{home}/.elan/bin/lean")]
        .into_iter()
        .find(|t| Command::new(t).arg("--version").output().is_ok())
}

pub fn translation(src: &str) -> Translation {
    let program = parse(src).unwrap();
    roop_check::check(&program).unwrap();
    translate(&program)
}

/// Runs Lean on the generated file. `Ok(())` when Lean accepts it or is not
/// installed; otherwise Lean's output and the file.
pub fn lean_accepts(t: &Translation) -> Result<(), String> {
    let Some(lean) = lean() else {
        return Ok(());
    };
    let dir = std::env::temp_dir().join(format!(
        "roop-lean-{}-{:x}",
        std::process::id(),
        t.lean.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Out.lean");
    std::fs::write(&file, &t.lean).unwrap();
    let out = Command::new(lean).arg(&file).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let errors: Vec<&str> = text.lines().filter(|l| l.contains(" error")).collect();
    if out.status.success() && errors.is_empty() {
        Ok(())
    } else {
        Err(format!("{text}\n--- file:\n{}", t.lean))
    }
}

/// Translates, asserts nothing was skipped, and has Lean check everything.
pub fn verified(src: &str) -> Translation {
    let t = translation(src);
    assert!(t.skipped.is_empty(), "skipped: {:?}", t.skipped);
    if let Err(report) = lean_accepts(&t) {
        panic!("{report}");
    }
    t
}
