use crate::world::{Chunk, world};

/// Writes `len` bytes of `buf` to descriptor `fd`, to be shown at the next
/// commit. Until then `roop_out_inv` takes them back.
///
/// # Safety
/// The pointers must be valid, `buf` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_out(fd: *const i64, buf: *const u8, len: *const i64) {
    let bytes = unsafe { std::slice::from_raw_parts(buf, (*len).max(0) as usize) };
    world().out.push(Chunk {
        fd: unsafe { *fd },
        bytes: bytes.to_vec(),
    });
}
