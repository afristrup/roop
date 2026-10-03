use std::path::{Path, PathBuf};

/// `dir/name.roop` or `dir/name/mod.roop`.
pub fn find_module_file(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let flat = dir.join(format!("{name}.roop"));
    let nested = dir.join(name).join("mod.roop");
    [&flat, &nested]
        .into_iter()
        .find(|p| p.is_file())
        .cloned()
        .ok_or_else(|| format!("{} or {}", flat.display(), nested.display()))
}
