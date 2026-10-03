use crate::io::ARGS;

/// The number of arguments, the program's name included.
///
/// # Safety
/// `count` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_arg_count(count: *mut i64) {
    unsafe { *count = ARGS.get().map_or(0, Vec::len) as i64 };
}
