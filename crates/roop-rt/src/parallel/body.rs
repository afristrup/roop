use std::ffi::c_void;

/// A generated loop body, called with the environment and the induction value.
pub type Body = extern "C" fn(*mut c_void, i64);
