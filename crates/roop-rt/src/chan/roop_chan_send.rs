use crate::chan::Channel;

/// Appends a copy of the message at `src`. Never blocks.
///
/// # Safety
/// `ch` must come from `roop_chan_new` and `src` must hold `elem` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_send(ch: *mut Channel, src: *const u8) {
    let channel = unsafe { &*ch };
    let message = unsafe { std::slice::from_raw_parts(src, channel.elem) }.to_vec();
    channel.queue.lock().unwrap().push_back(message);
    channel.ready.notify_one();
}
