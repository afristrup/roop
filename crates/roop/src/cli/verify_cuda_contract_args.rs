use std::path::PathBuf;

pub struct VerifyCudaContractArgs {
    pub ptx: PathBuf,
    pub contract: PathBuf,
}
