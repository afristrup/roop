use crate::chan::Channel;
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

/// Creates a channel for messages of `elem_size` bytes.
#[unsafe(no_mangle)]
pub extern "C" fn roop_chan_new(elem_size: i64) -> *mut Channel {
    Box::into_raw(Box::new(Channel {
        elem: elem_size as usize,
        queue: Mutex::new(VecDeque::new()),
        ready: Condvar::new(),
    }))
}
