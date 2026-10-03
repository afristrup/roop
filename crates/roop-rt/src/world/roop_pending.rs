use crate::world::{require_zero, world};

/// How many bytes of output are written and not shown yet.
///
/// # Safety
/// `len` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_pending(len: *mut i64) {
    unsafe { require_zero(len as *const u8, 8, "the length") };
    unsafe { *len = world().out.iter().map(|c| c.bytes.len() as i64).sum() };
}
