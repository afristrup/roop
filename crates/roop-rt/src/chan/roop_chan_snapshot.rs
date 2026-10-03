use crate::chan::{Channel, Snapshot};

/// Copies the queued messages, so a rollback can put them back.
///
/// # Safety
/// `ch` must come from `roop_chan_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_snapshot(ch: *mut Channel) -> *mut Snapshot {
    let queue = unsafe { &*ch }.queue.lock().unwrap().clone();
    Box::into_raw(Box::new(Snapshot(queue)))
}
