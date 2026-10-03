use crate::io::{path_of, status_of};

/// Renames a file or directory; `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, each path for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_rename(
    from: *const u8,
    from_len: *const i64,
    to: *const u8,
    to_len: *const i64,
    status: *mut i64,
) {
    let (from, to) = unsafe { (path_of(from, *from_len), path_of(to, *to_len)) };
    unsafe { *status = status_of(std::fs::rename(from, to)) };
}
