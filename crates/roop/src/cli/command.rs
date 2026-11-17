use crate::{
    BuildArgs, FmtArgs, HybridArgs, LeanArgs, RunArgs, TestArgs, VerifyCudaContractArgs, WeaveArgs,
};

pub enum Command {
    Build(BuildArgs),
    Lean(LeanArgs),
    Fmt(FmtArgs),
    Test(TestArgs),
    Run(RunArgs),
    Weave(WeaveArgs),
    Hybrid(HybridArgs),
    VerifyCudaContract(VerifyCudaContractArgs),
    Lsp,
}
