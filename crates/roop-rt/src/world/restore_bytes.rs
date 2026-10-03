use crate::world::refuse;
use std::path::Path;

/// Puts a file back as it was: the bytes, or gone.
pub fn restore_bytes(path: &Path, old: &Option<Vec<u8>>) {
    let result = match old {
        Some(bytes) => std::fs::write(path, bytes),
        None => std::fs::remove_file(path),
    };
    if let Err(e) = result {
        refuse(&format!("cannot restore {}: {e}", path.display()));
    }
}
