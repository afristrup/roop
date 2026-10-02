use crate::{BuildArgs, LeanArgs};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
}
