use crate::io::ARGS;
use crate::world::require_zero;

/// Copies argument `index` into `buf`, at most `cap` bytes. `len` gets its full
/// length, or -1 when there is no such argument. The results start zero.
///
/// # Safety
/// The pointers must be valid, `buf` for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_arg(index: *const i64, buf: *mut u8, cap: *const i64, len: *mut i64) {
    let cap = unsafe { *cap }.max(0) as usize;
    unsafe {
        require_zero(buf, cap, "the buffer");
        require_zero(len as *const u8, 8, "the length");
    }
    let arg = ARGS
        .get()
        .and_then(|a| a.get(unsafe { *index }.max(0) as usize));
    let Some(arg) = arg else {
        unsafe { *len = -1 };
        return;
    };
    let n = arg.len().min(cap);
    unsafe { std::ptr::copy_nonoverlapping(arg.as_ptr(), buf, n) };
    unsafe { *len = arg.len() as i64 };
}
