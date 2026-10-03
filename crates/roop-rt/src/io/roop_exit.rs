use crate::world::roop_commit;

/// Shows what the program has written, then ends it with `code`.
///
/// # Safety
/// `code` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_exit(code: *const i64) {
    roop_commit();
    std::process::exit(unsafe { *code } as i32);
}
