use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

/// An unbounded FIFO of fixed-size messages between two tasks.
pub struct Channel {
    pub elem: usize,
    pub queue: Mutex<VecDeque<Vec<u8>>>,
    pub ready: Condvar,
}
