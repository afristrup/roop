use crate::{Dir, Kernel, KernelArg, KernelBufferRole};

/// A compact description of one emitted device kernel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelContract {
    pub kernel: String,
    pub direction: &'static str,
    pub induction: String,
    pub buffers: Vec<KernelBufferContract>,
}

/// A device buffer visible to an emitted kernel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelBufferContract {
    /// Zero-based position in the emitted kernel parameter list.
    pub slot: usize,
    pub name: String,
    pub role: &'static str,
    pub llvm_ty: String,
    pub space: u32,
    pub bytes: u64,
    pub align: u64,
    pub writable: bool,
}

impl KernelContract {
    pub fn from_kernel(kernel: &Kernel) -> Self {
        let buffers = kernel
            .args
            .iter()
            .enumerate()
            .filter_map(|(slot, arg)| match arg {
                KernelArg::Buffer {
                    name,
                    role,
                    llvm_ty,
                    size,
                    align,
                    writable,
                    space,
                    ..
                } => Some(KernelBufferContract {
                    slot,
                    name: name.clone(),
                    role: match role {
                        KernelBufferRole::Capture => "capture",
                        KernelBufferRole::LaunchParams => "launch_params",
                        KernelBufferRole::Error => "error",
                    },
                    llvm_ty: llvm_ty.clone(),
                    space: *space,
                    bytes: *size,
                    align: *align,
                    writable: *writable,
                }),
                KernelArg::ThreadId => None,
            })
            .collect();
        Self {
            kernel: kernel.name.clone(),
            direction: match kernel.direction {
                Dir::Forward => "forward",
                Dir::Backward => "backward",
            },
            induction: kernel.induction.clone(),
            buffers,
        }
    }
}
