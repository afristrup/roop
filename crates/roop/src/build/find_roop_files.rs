use crate::CliError;
use std::path::{Path, PathBuf};

/// Every `.roop` file at or under `path`, in order. Directories that start
/// with a dot and `target` are skipped.
pub fn find_roop_files(path: &Path) -> Result<Vec<PathBuf>, CliError> {
    let io = |e| CliError::Io(path.display().to_string(), e);
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
        .map_err(io)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .map_err(io)?;
    entries.sort();
    let mut found = Vec::new();
    for entry in entries {
        let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if entry.is_dir() && !name.starts_with('.') && name != "target" {
            found.extend(find_roop_files(&entry)?);
        } else if entry.extension().is_some_and(|e| e == "roop") {
            found.push(entry);
        }
    }
    Ok(found)
}
