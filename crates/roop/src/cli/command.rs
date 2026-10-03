use crate::{BuildArgs, FmtArgs, LeanArgs};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
    Fmt(FmtArgs),
}
