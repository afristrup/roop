use crate::io::path_of;
use crate::world::{Entry, errno_of, file_bytes, journal, require_zero};

/// Makes a file hold the first `blen` bytes of `buf`, replacing it. What it held
/// is kept, for `roop_fs_write_inv`. `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, each for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_write(
    path: *const u8,
    len: *const i64,
    buf: *const u8,
    blen: *const i64,
    status: *mut i64,
) {
    unsafe { require_zero(status as *const u8, 8, "the status") };
    let path = unsafe { path_of(path, *len) };
    let new = unsafe { std::slice::from_raw_parts(buf, (*blen).max(0) as usize) }.to_vec();
    let old = file_bytes(&path);
    match std::fs::write(&path, &new) {
        Ok(()) => journal(Entry::Wrote { path, old, new }),
        Err(e) => {
            journal(Entry::Failed);
            unsafe { *status = errno_of(&e) };
        }
    }
}
