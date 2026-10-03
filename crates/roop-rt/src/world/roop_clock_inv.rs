use crate::world::{refuse, world};

/// # Safety
/// `ms` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_clock_inv(ms: *mut i64) {
    let mut world = world();
    let Some(read) = world.clock_consumed.pop() else {
        refuse("there is no time to put back");
    };
    world.clock_ahead.push_front(read);
    unsafe { *ms = 0 };
}
