use crate::gpu::{OK, RoopBuf, UNAVAILABLE, metal_dispatch};
use std::ffi::{CStr, c_char, c_void};

/// Runs a generated kernel on a GPU. `kind` 0 is Metal (`blob` is a
/// `.metallib`), 1 is CUDA (`blob` is NUL-terminated PTX).
///
/// # Safety
/// Pointers must come from generated code: `bufs` holds `nbufs` valid
/// descriptors and `kernel` is a NUL-terminated name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_gpu_dispatch(
    kind: i32,
    blob: *const c_void,
    blob_len: i64,
    kernel: *const c_char,
    bufs: *const RoopBuf,
    nbufs: i64,
    lo: i64,
    step: i64,
    count: i64,
) -> i32 {
    let (kernel, bufs) = unsafe {
        (
            CStr::from_ptr(kernel).to_string_lossy().into_owned(),
            std::slice::from_raw_parts(bufs, nbufs as usize),
        )
    };
    let status = match kind {
        0 => {
            let blob = unsafe { std::slice::from_raw_parts(blob as *const u8, blob_len as usize) };
            metal_dispatch(blob, &kernel, bufs, [lo, step, count])
        }
        _ => UNAVAILABLE,
    };
    if status == UNAVAILABLE {
        eprintln!("roop: no GPU backend available for kernel `{kernel}`");
    }
    let _ = OK;
    status
}
