use crate::world::world;

/// The most bytes the history of kept values has held so far.
///
/// # Safety
/// `peak` must be valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_history_peak(peak: *mut i64) {
    unsafe { *peak = world().kept_peak as i64 };
}
