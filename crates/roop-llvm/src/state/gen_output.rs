use crate::Kernel;

/// A generated host function with its outlined helpers, plus the device
/// kernels its parallel loops produced.
pub struct GenOutput {
    pub text: String,
    pub kernels: Vec<Kernel>,
}
