use std::fs::File;
use std::mem::ManuallyDrop;
use std::os::fd::FromRawFd;

/// Runs `f` on the file with descriptor `fd` without taking ownership of it.
///
/// # Safety
/// `fd` must be an open descriptor.
pub unsafe fn with_fd<T>(fd: i64, f: impl FnOnce(&mut File) -> T) -> T {
    let mut file = ManuallyDrop::new(unsafe { File::from_raw_fd(fd as i32) });
    f(&mut file)
}
