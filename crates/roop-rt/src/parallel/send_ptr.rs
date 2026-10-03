use std::ffi::c_void;

/// The compiler proves iterations touch disjoint data, so sharing the
/// environment pointer across threads is sound for generated loops.
#[derive(Clone, Copy)]
pub struct SendPtr(pub *mut c_void);

unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}
