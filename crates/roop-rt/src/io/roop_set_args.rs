use crate::io::ARGS;
use std::ffi::{CStr, c_char, c_int};

/// Records `argc` and `argv` for `roop_arg`.
///
/// # Safety
/// `argv` must hold `argc` C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_set_args(argc: c_int, argv: *const *const c_char) {
    let args = (0..argc.max(0) as usize)
        .map(|i| unsafe { CStr::from_ptr(*argv.add(i)) }.to_bytes().to_vec())
        .collect();
    let _ = ARGS.set(args);
}
