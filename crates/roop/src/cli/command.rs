use crate::{BuildArgs, FmtArgs, LeanArgs, RunArgs, TestArgs};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
    Fmt(FmtArgs),
    Test(TestArgs),
    Run(RunArgs),
}
