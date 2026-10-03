use crate::chan::Snapshot;

/// # Safety
/// `snapshot` must come from `roop_chan_snapshot` and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_snapshot_free(snapshot: *mut Snapshot) {
    drop(unsafe { Box::from_raw(snapshot) });
}
