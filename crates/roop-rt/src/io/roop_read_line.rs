use crate::io::errno;
use std::io::BufRead;

/// Reads a line of standard input into `buf`, at most `cap` bytes, without the
/// newline. `got` is the number of bytes stored. `status` is 0, 1 at the end of
/// the input when nothing was read, or a negative error. A longer line is cut
/// off and its rest is skipped.
///
/// # Safety
/// The pointers must be valid, `buf` for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_read_line(
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
    status: *mut i64,
) {
    let mut line = Vec::new();
    let read = std::io::stdin().lock().read_until(b'\n', &mut line);
    let (count, code) = match read {
        Ok(0) => (0, 1),
        Ok(_) => {
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let cap = unsafe { *cap }.max(0) as usize;
            unsafe { std::ptr::write_bytes(buf, 0, cap) };
            let n = line.len().min(cap);
            unsafe { std::ptr::copy_nonoverlapping(line.as_ptr(), buf, n) };
            (n as i64, 0)
        }
        Err(e) => (0, errno(&e)),
    };
    unsafe {
        *got = count;
        *status = code;
    }
}
