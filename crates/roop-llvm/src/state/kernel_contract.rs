use crate::{Dir, Kernel, KernelArg};

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
    pub name: String,
    pub bytes: u64,
    pub align: u64,
    pub writable: bool,
}

impl KernelContract {
    pub fn from_kernel(kernel: &Kernel) -> Self {
        let buffers = kernel
            .args
            .iter()
            .filter_map(|arg| match arg {
                KernelArg::Buffer {
                    name,
                    size,
                    align,
                    writable,
                    ..
                } => Some(KernelBufferContract {
                    name: name.clone(),
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
