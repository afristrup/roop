use std::fs::File;
use std::os::fd::FromRawFd;

/// Closes a descriptor from `roop_open`.
///
/// # Safety
/// `fd` must be open, and not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_close(fd: *const i64) {
    drop(unsafe { File::from_raw_fd(*fd as i32) });
}
