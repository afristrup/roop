use std::path::{Path, PathBuf};

/// Where the submodules of `file` live: next to it for `mod.roop` and for the
/// entry file, in a directory named after it otherwise.
pub fn child_dir(file: &Path, is_entry: bool) -> PathBuf {
    let parent = file.parent().unwrap_or(Path::new("."));
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if is_entry || stem == "mod" {
        parent.to_path_buf()
    } else {
        parent.join(stem)
    }
}
