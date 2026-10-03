use crate::world::HISTORY_LIMIT;
use std::sync::atomic::Ordering;

/// Sets the most bytes the history of kept values may hold, from `Roop.toml`.
#[unsafe(no_mangle)]
pub extern "C" fn roop_set_history_limit(bytes: u64) {
    HISTORY_LIMIT.store(bytes, Ordering::Relaxed);
}
