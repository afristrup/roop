use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// The path made of the `len` bytes at `ptr`, up to the first zero byte, so a
/// zero-padded buffer works as a path.
///
/// # Safety
/// `ptr` must point at `len` readable bytes.
pub unsafe fn path_of(ptr: *const u8, len: i64) -> PathBuf {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len.max(0) as usize) };
    let bytes = bytes.split(|b| *b == 0).next().unwrap_or_default();
    PathBuf::from(OsStr::from_bytes(bytes))
}
