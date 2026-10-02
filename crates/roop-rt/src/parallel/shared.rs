use crate::parallel::PoolState;
use std::sync::{Condvar, Mutex};

pub struct Shared {
    pub state: Mutex<PoolState>,
    pub wake: Condvar,
    pub done: Condvar,
}
