use crate::{BuildArgs, FmtArgs, LeanArgs, RunArgs, TestArgs, WeaveArgs};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
    Fmt(FmtArgs),
    Test(TestArgs),
    Run(RunArgs),
    Weave(WeaveArgs),
    Lsp,
}
