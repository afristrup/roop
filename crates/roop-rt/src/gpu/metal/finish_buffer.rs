#![cfg(target_os = "macos")]

use crate::gpu::metal::{Staged, return_pooled};
use objc2_metal::MTLBuffer;

/// Copies a written staging buffer back to the caller and recycles it.
/// Zero-copy buffers already hold the result in the caller's memory.
///
/// # Safety
/// `host` must be valid for `staged.size` bytes.
pub unsafe fn finish_buffer(staged: Staged, host: *mut u8, written: bool) {
    if staged.zero_copy {
        return;
    }
    if written {
        unsafe {
            std::ptr::copy_nonoverlapping(
                staged.buffer.contents().as_ptr().cast(),
                host,
                staged.size,
            )
        };
    }
    return_pooled(staged.size, staged.buffer);
}
