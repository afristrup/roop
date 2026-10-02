#![cfg(target_os = "macos")]

use crate::gpu::count_zero_copy;
use crate::gpu::metal::{DeviceState, Staged, page_size, take_pooled};
use objc2_metal::{MTLBuffer, MTLDevice, MTLResourceOptions};
use std::ptr::NonNull;

/// Makes `size` bytes at `host` visible to the GPU. Page-aligned memory is
/// shared directly (unified memory, no copy); anything else goes through a
/// pooled staging buffer that is filled from `host`.
///
/// # Safety
/// `host` must be valid for `size` bytes until the kernel finishes.
pub unsafe fn stage_buffer(state: &DeviceState, host: *mut u8, size: usize) -> Option<Staged> {
    let page = page_size();
    let options = MTLResourceOptions::StorageModeShared;
    if host as usize % page == 0 && size % page == 0 && size > 0 {
        let pointer = NonNull::new(host.cast())?;
        let buffer = unsafe {
            state
                .device
                .newBufferWithBytesNoCopy_length_options_deallocator(pointer, size, options, None)
        };
        if let Some(buffer) = buffer {
            count_zero_copy();
            return Some(Staged {
                buffer,
                size,
                zero_copy: true,
            });
        }
    }
    let buffer = match take_pooled(size) {
        Some(buffer) => buffer,
        None => state
            .device
            .newBufferWithLength_options(size.max(1), options)?,
    };
    unsafe { std::ptr::copy_nonoverlapping(host, buffer.contents().as_ptr().cast(), size) };
    Some(Staged {
        buffer,
        size,
        zero_copy: false,
    })
}
