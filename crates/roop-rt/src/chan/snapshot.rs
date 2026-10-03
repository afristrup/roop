use std::collections::VecDeque;

/// The queued messages of a channel at one moment, for rollback.
pub struct Snapshot(pub VecDeque<Vec<u8>>);
