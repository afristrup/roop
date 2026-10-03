#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Forward,
    Backward,
}

impl Dir {
    pub fn flip(self) -> Dir {
        match self {
            Dir::Forward => Dir::Backward,
            Dir::Backward => Dir::Forward,
        }
    }
}
