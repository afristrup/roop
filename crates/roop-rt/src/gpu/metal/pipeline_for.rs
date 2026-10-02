#![cfg(target_os = "macos")]

use crate::gpu::metal::DeviceState;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{NSString, NSURL};
use objc2_metal::{MTLComputePipelineState, MTLDevice, MTLLibrary};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Loads the kernel from the embedded metallib. Metal reads libraries from
/// files, so the blob is written to the temp directory once.
pub fn pipeline_for(
    state: &DeviceState,
    blob: &[u8],
    kernel: &str,
) -> Option<Retained<ProtocolObject<dyn MTLComputePipelineState>>> {
    let mut hasher = DefaultHasher::new();
    blob.hash(&mut hasher);
    let path = std::env::temp_dir().join(format!("roop-{:016x}.metallib", hasher.finish()));
    if !path.exists() {
        std::fs::write(&path, blob).ok()?;
    }
    let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str()?));
    let library = state.device.newLibraryWithURL_error(&url).ok()?;
    let function = library.newFunctionWithName(&NSString::from_str(kernel))?;
    state
        .device
        .newComputePipelineStateWithFunction_error(&function)
        .ok()
}
