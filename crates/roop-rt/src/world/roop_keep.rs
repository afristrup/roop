use crate::world::{Kept, world};

/// Hands `size` bytes to the world, which remembers them, and zeroes them. This
/// is how a reversible program lets go of a value without losing it: the
/// history lives in the world until the process ends.
///
/// # Safety
/// `ptr` must be valid for `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_keep(ptr: *mut u8, size: i64) {
    let size = size.max(0) as usize;
    let bytes = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
    let used = bytes
        .iter()
        .rposition(|b| *b != 0)
        .map_or(0, |last| last + 1);
    world().kept.push(Kept {
        size,
        bytes: bytes[..used].to_vec(),
    });
    bytes.fill(0);
}
