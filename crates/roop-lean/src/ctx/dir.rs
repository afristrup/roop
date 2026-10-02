/// Which way a block is being translated: as written, or its inverse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Forward,
    Backward,
}
