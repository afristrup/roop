use crate::chan::Channel;

/// The number of messages waiting.
///
/// # Safety
/// `ch` must come from `roop_chan_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_len(ch: *mut Channel) -> i64 {
    unsafe { &*ch }.queue.lock().unwrap().len() as i64
}
