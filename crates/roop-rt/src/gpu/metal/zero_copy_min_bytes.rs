#![cfg(target_os = "macos")]

use std::sync::OnceLock;

/// Below this size, copying through a pooled buffer is faster than asking
/// Metal to wrap the caller's memory (measured on an M4: 8 MiB arrays lose,
/// 32 MiB arrays win). `ROOP_ZERO_COPY_MIN_BYTES` overrides it.
const DEFAULT: usize = 16 << 20;

pub fn zero_copy_min_bytes() -> usize {
    static MIN: OnceLock<usize> = OnceLock::new();
    *MIN.get_or_init(|| {
        std::env::var("ROOP_ZERO_COPY_MIN_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT)
    })
}
