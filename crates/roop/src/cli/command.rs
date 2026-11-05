use crate::{BuildArgs, FmtArgs, LeanArgs, RunArgs, TestArgs, VerifyCudaContractArgs, WeaveArgs};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
    Fmt(FmtArgs),
    Test(TestArgs),
    Run(RunArgs),
    Weave(WeaveArgs),
    VerifyCudaContract(VerifyCudaContractArgs),
    Lsp,
}
