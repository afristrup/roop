/// # Safety
/// `count` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_arg_count_inv(count: *mut i64) {
    unsafe { *count = 0 };
}
