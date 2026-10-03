use crate::io::errno;

/// A count for success, the failure as a negative number otherwise.
pub fn count_of(result: std::io::Result<usize>) -> i64 {
    result.map_or_else(|e| errno(&e), |n| n as i64)
}
