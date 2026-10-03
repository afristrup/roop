use crate::parallel::{Body, SendPtr};

/// One parallel loop handed to the pool. `next` points at the caller's
/// block counter, which outlives the job because the caller waits for it.
#[derive(Clone, Copy)]
pub struct Job {
    pub lo: i64,
    pub step: i64,
    pub count: i64,
    pub block: i64,
    pub body: Body,
    pub env: SendPtr,
    pub next: SendPtr,
}
