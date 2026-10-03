use crate::chan::{Channel, Snapshot};

/// Replaces the channel's queue with the snapshot's messages.
///
/// # Safety
/// Both pointers must be live and come from this runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_restore(ch: *mut Channel, snapshot: *const Snapshot) {
    let queue = unsafe { &*snapshot }.0.clone();
    *unsafe { &*ch }.queue.lock().unwrap() = queue;
}
