use crate::world::{require_zero, world};
use std::time::{SystemTime, UNIX_EPOCH};

/// The time in milliseconds since 1970. What was read is kept, so that after
/// `roop_clock_inv` the next read gives the same time again, as a replay of the
/// run would.
///
/// # Safety
/// `ms` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_clock(ms: *mut i64) {
    unsafe { require_zero(ms as *const u8, 8, "the time") };
    let mut world = world();
    let now = world.clock_ahead.pop_front().unwrap_or_else(|| {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        elapsed.as_millis() as i64
    });
    world.clock_consumed.push(now);
    unsafe { *ms = now };
}
