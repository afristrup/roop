use crate::world::{HISTORY_LIMIT, Kept, refuse, world};
use std::sync::atomic::Ordering;

/// Hands `size` bytes to the world, which remembers them, and zeroes them. This
/// is how a reversible program lets go of a value without losing it: the
/// history lives in the world until the process ends, or `roop_forget`, and
/// stops the program if it would pass the limit in `Roop.toml`.
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
    let kept = Kept {
        size,
        bytes: bytes[..used].to_vec(),
    };
    let mut world = world();
    let limit = HISTORY_LIMIT.load(Ordering::Relaxed);
    if limit != 0 && (world.kept_bytes + kept.cost()) as u64 > limit {
        refuse(&format!(
            "the history of kept values passed its limit of {limit} bytes; let it go with std::process::forget, or raise history_limit in Roop.toml"
        ));
    }
    world.kept_bytes += kept.cost();
    world.kept_peak = world.kept_peak.max(world.kept_bytes);
    world.kept.push(kept);
    bytes.fill(0);
}
