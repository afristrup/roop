use crate::world::{Input, next_input, require_zero, world};

/// Reads a line of standard input into `buf`, without its newline and cut to
/// `cap` bytes. `got` is the number of bytes; `status` is 0, 1 at the end of the
/// input, or a negative error. The results start zero. `roop_in_line_inv` puts
/// the line back to be read again.
///
/// # Safety
/// The pointers must be valid, `buf` for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_in_line(
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
    status: *mut i64,
) {
    let cap = unsafe { *cap }.max(0) as usize;
    unsafe {
        require_zero(buf, cap, "the buffer");
        require_zero(got as *const u8, 8, "the length");
        require_zero(status as *const u8, 8, "the status");
    }
    let mut world = world();
    let input = next_input(&mut world);
    let (count, code) = match &input {
        Input::Line(line) => {
            let n = line.len().min(cap);
            unsafe { std::ptr::copy_nonoverlapping(line.as_ptr(), buf, n) };
            (n as i64, 0)
        }
        Input::Eof => (0, 1),
        Input::Failed(e) => (0, *e),
    };
    unsafe {
        *got = count;
        *status = code;
    }
    world.consumed.push(input);
}
