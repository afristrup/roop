use crate::io::errno;

/// 0 for success, the failure otherwise.
pub fn status_of(result: std::io::Result<()>) -> i64 {
    result.map_or_else(|e| errno(&e), |()| 0)
}
