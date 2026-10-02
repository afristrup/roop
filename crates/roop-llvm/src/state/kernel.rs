use crate::{Dialect, KernelArg};

#[derive(Clone, Debug)]
pub struct Kernel {
    pub dialect: Dialect,
    pub name: String,
    /// The complete `define` for the kernel.
    pub text: String,
    /// LLVM parameter types in order, for the function type in metadata.
    pub param_types: Vec<String>,
    pub args: Vec<KernelArg>,
}
