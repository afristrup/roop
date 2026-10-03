use crate::world::{refuse, world, zero_bytes};

/// Puts the last line read back: the next read gets it again.
///
/// # Safety
/// The pointers must be valid, `buf` for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_in_line_inv(
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
    status: *mut i64,
) {
    let mut world = world();
    let Some(input) = world.consumed.pop() else {
        refuse("there is no input to put back");
    };
    world.ahead.push_front(input);
    unsafe {
        zero_bytes(buf, (*cap).max(0) as usize);
        *got = 0;
        *status = 0;
    }
}
