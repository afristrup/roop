#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLBuffer;

/// A buffer the kernel will use. `zero_copy` buffers alias the caller's
/// memory; others are pooled staging buffers that need copying.
pub struct Staged {
    pub buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub size: usize,
    pub zero_copy: bool,
}
