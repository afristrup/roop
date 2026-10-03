use std::sync::atomic::AtomicU64;

/// The most bytes the history of kept values may hold; zero means no limit.
pub static HISTORY_LIMIT: AtomicU64 = AtomicU64::new(0);
