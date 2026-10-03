#![cfg(target_os = "macos")]

unsafe extern "C" {
    fn getpagesize() -> i32;
}

pub fn page_size() -> usize {
    unsafe { getpagesize() as usize }
}
