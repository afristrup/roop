use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since 1970.
///
/// # Safety
/// `ms` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_now_ms(ms: *mut i64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    unsafe { *ms = now.as_millis() as i64 };
}
