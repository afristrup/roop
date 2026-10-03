use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// One thing the program read from outside: a line, the end of the input, or
/// the error that stopped the read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Line(Vec<u8>),
    Eof,
    Failed(i64),
}

/// A value the program gave to the world with `keep`: its size, and its bytes
/// up to the last one that is not zero, since a buffer is mostly zeros.
pub struct Kept {
    pub size: usize,
    pub bytes: Vec<u8>,
}

impl Kept {
    /// What the entry costs against the history limit: its bytes and its size.
    pub fn cost(&self) -> usize {
        self.bytes.len() + 2 * std::mem::size_of::<usize>()
    }
}

/// Output the program has written and the world has not shown yet.
pub struct Chunk {
    pub fd: i64,
    pub bytes: Vec<u8>,
}

/// What a file operation changed, kept so its inverse can put it back.
pub enum Entry {
    /// The operation failed and changed nothing.
    Failed,
    Wrote {
        path: PathBuf,
        old: Option<Vec<u8>>,
        new: Vec<u8>,
    },
    Appended {
        path: PathBuf,
        old_len: Option<u64>,
        added: Vec<u8>,
    },
    Removed {
        path: PathBuf,
        old: Vec<u8>,
    },
    MadeDirs {
        created: Vec<PathBuf>,
    },
    RemovedDir {
        path: PathBuf,
    },
    Renamed {
        from: PathBuf,
        to: PathBuf,
        overwritten: Option<Vec<u8>>,
    },
}

/// The world as the program sees it. Everything here is what lets a function
/// that touches the world be run backward: output that is not shown yet can be
/// taken back, input that was read can be put back, to be read again, and a
/// file operation leaves the state it replaced.
pub struct World {
    pub out: Vec<Chunk>,
    pub ahead: VecDeque<Input>,
    pub consumed: Vec<Input>,
    pub clock_ahead: VecDeque<i64>,
    pub clock_consumed: Vec<i64>,
    pub journal: Vec<Entry>,
    pub kept: Vec<Kept>,
    /// The bytes the kept values hold, counted against the history limit.
    pub kept_bytes: usize,
}

static WORLD: Mutex<World> = Mutex::new(World {
    out: Vec::new(),
    ahead: VecDeque::new(),
    consumed: Vec::new(),
    clock_ahead: VecDeque::new(),
    clock_consumed: Vec::new(),
    journal: Vec::new(),
    kept: Vec::new(),
    kept_bytes: 0,
});

/// The world, locked for the length of one operation.
pub fn world() -> MutexGuard<'static, World> {
    WORLD.lock().unwrap_or_else(|e| e.into_inner())
}
