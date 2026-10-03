/// # Safety
/// `len` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_pending_inv(len: *mut i64) {
    unsafe { *len = 0 };
}
