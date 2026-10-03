use crate::{CliError, find_roop_files};
use std::path::{Path, PathBuf};

/// The files of the given paths, or of the project, that declare a test.
pub fn test_files(paths: &[PathBuf], root: &Path) -> Result<Vec<PathBuf>, CliError> {
    let roots: Vec<&Path> = match paths.is_empty() {
        true => vec![root],
        false => paths.iter().map(PathBuf::as_path).collect(),
    };
    let mut found = Vec::new();
    for root in roots {
        for file in find_roop_files(root)? {
            let name = file.display().to_string();
            let src = std::fs::read_to_string(&file).map_err(|e| CliError::Io(name.clone(), e))?;
            if declares_test(&src) {
                found.push(file);
            }
        }
    }
    Ok(found)
}

/// A cheap look for a line that starts a test, before the file is parsed.
fn declares_test(src: &str) -> bool {
    src.lines().any(|line| {
        let mut words = line.split_whitespace();
        words.next() == Some("test") && words.next().is_some()
    })
}
