use crate::world::{refuse, world};

/// Takes back the last output, which must be these bytes and not shown yet.
///
/// # Safety
/// The pointers must be valid, `buf` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_out_inv(fd: *const i64, buf: *const u8, len: *const i64) {
    let bytes = unsafe { std::slice::from_raw_parts(buf, (*len).max(0) as usize) };
    let Some(chunk) = world().out.pop() else {
        refuse("cannot take back output that was already shown");
    };
    if chunk.fd != unsafe { *fd } || chunk.bytes != bytes {
        refuse("the output taken back is not the output that was written last");
    }
}
