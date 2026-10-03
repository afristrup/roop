/// An I/O failure as a negative number: minus the operating system's error code,
/// or minus 5 when it gave none.
pub fn errno_of(error: &std::io::Error) -> i64 {
    -i64::from(error.raw_os_error().unwrap_or(5))
}
