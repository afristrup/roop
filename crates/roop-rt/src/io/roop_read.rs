use crate::io::{count_of, with_fd};
use std::io::Read;

/// Reads up to `cap` bytes from descriptor `fd` into `buf` starting at byte
/// `offset`; `got` gets the count, 0 at the end of the file, or a negative
/// error.
///
/// # Safety
/// The pointers must be valid, `buf` for `offset + cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_read(
    fd: *const i64,
    buf: *mut u8,
    offset: *const i64,
    cap: *const i64,
    got: *mut i64,
) {
    let cap = unsafe { *cap }.max(0) as usize;
    let bytes = unsafe { std::slice::from_raw_parts_mut(buf.add(*offset as usize), cap) };
    let result = unsafe { with_fd(*fd, |f| f.read(bytes)) };
    unsafe { *got = count_of(result) };
}
