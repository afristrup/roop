use crate::{BuildArgs, FmtArgs, LeanArgs, TestArgs};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
    Fmt(FmtArgs),
    Test(TestArgs),
}
