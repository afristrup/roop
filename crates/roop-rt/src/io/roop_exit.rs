/// Ends the process with `code`.
///
/// # Safety
/// `code` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_exit(code: *const i64) {
    std::process::exit(unsafe { *code } as i32);
}
