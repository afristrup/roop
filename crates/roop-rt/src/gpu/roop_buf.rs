use std::ffi::c_void;

/// One captured variable as the generated host code describes it.
#[repr(C)]
pub struct RoopBuf {
    pub ptr: *mut c_void,
    pub size: i64,
    pub written: i32,
}
