/// Puts the target back as it was before the copy.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_copy_inv(
    source: *const u8,
    source_len: *const i64,
    target: *const u8,
    target_len: *const i64,
    status: *mut i64,
) {
    // A copy leaves the same journal entry as a write.
    unsafe { crate::world::roop_fs_write_inv(source, source_len, target, target_len, status) };
}
