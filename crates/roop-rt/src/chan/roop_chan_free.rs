use crate::chan::Channel;

/// # Safety
/// `ch` must come from `roop_chan_new` and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_free(ch: *mut Channel) {
    drop(unsafe { Box::from_raw(ch) });
}
