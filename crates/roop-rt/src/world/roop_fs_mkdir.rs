use crate::io::path_of;
use crate::world::{Entry, errno_of, journal, require_zero};

/// Makes a directory and the ones above it that are missing, keeping which
/// those were. `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_mkdir(path: *const u8, len: *const i64, status: *mut i64) {
    unsafe { require_zero(status as *const u8, 8, "the status") };
    let path = unsafe { path_of(path, *len) };
    let mut created: Vec<_> = path
        .ancestors()
        .filter(|p| !p.as_os_str().is_empty() && !p.exists())
        .map(|p| p.to_path_buf())
        .collect();
    created.reverse();
    match std::fs::create_dir_all(&path) {
        Ok(()) => journal(Entry::MadeDirs { created }),
        Err(e) => {
            journal(Entry::Failed);
            unsafe { *status = errno_of(&e) };
        }
    }
}
