use std::path::Path;

/// What a file holds, or none when there is no such file.
pub fn file_bytes(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}
