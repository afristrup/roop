use crate::{CliError, find_lean, run_tool};
use roop_syntax::Program;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// How the Lean model compares with the compiled tests.
#[derive(Default)]
pub struct Agreement {
    pub agree: usize,
    /// Tests where the model and the compiled program differ.
    pub differ: Vec<String>,
    /// Tests the model does not cover.
    pub unmodelled: Vec<String>,
}

/// Runs every test on the Lean model of `program` as well, and compares the
/// verdict with `native`, the verdict of the compiled test. A disagreement is
/// a bug in the compiler or in the model.
pub fn lean_agreement(
    program: &Program,
    native: &[(String, bool)],
    dir: &Path,
) -> Result<Agreement, CliError> {
    let model = roop_lean::translate_models(program);
    let mut lean = model.lean.clone();
    lean.push_str(&roop_lean::test_checks(program, &model));
    let file = dir.join("model.lean");
    std::fs::write(&file, lean).map_err(|e| CliError::Io(file.display().to_string(), e))?;
    let exe = find_lean().ok_or_else(|| CliError::Tool("lean not found".into()))?;
    let mut command = Command::new(exe);
    command.arg(&file);
    let printed = String::from_utf8_lossy(&run_tool(command)?).into_owned();
    let verdicts: HashMap<&str, bool> = printed
        .lines()
        .filter_map(|line| {
            let mut parts = line.strip_prefix("@lean ")?.split_whitespace();
            Some((parts.next()?, parts.next()? == "true"))
        })
        .collect();
    let mut agreement = Agreement::default();
    for (name, passed) in native {
        match verdicts.get(name.as_str()) {
            None => agreement.unmodelled.push(name.clone()),
            Some(lean) if lean == passed => agreement.agree += 1,
            Some(_) => agreement.differ.push(name.clone()),
        }
    }
    Ok(agreement)
}
