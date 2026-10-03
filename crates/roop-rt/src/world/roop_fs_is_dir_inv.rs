/// # Safety
/// `out` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_is_dir_inv(_path: *const u8, _len: *const i64, out: *mut i64) {
    unsafe { *out = 0 };
}
