use crate::chan::Channel;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

/// Waits for a message and copies it to `dst`. Returns 0, or 1 when `abort`
/// (a flag a failing task sets; may be null) was raised while waiting.
///
/// # Safety
/// `ch` must come from `roop_chan_new`; `dst` must hold `elem` bytes; `abort`
/// is null or points at a live `i32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_chan_recv(ch: *mut Channel, dst: *mut u8, abort: *const i32) -> i32 {
    let channel = unsafe { &*ch };
    let aborted = || {
        !abort.is_null()
            && unsafe { AtomicI32::from_ptr(abort as *mut i32) }.load(Ordering::Relaxed) != 0
    };
    let mut queue = channel.queue.lock().unwrap();
    loop {
        if let Some(message) = queue.pop_front() {
            unsafe { std::ptr::copy_nonoverlapping(message.as_ptr(), dst, channel.elem) };
            return 0;
        }
        if aborted() {
            return 1;
        }
        queue = channel
            .ready
            .wait_timeout(queue, Duration::from_millis(1))
            .unwrap()
            .0;
    }
}
