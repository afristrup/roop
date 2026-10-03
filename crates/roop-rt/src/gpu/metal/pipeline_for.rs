#![cfg(target_os = "macos")]

use crate::gpu::metal::DeviceState;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{NSString, NSURL};
use objc2_metal::{MTLComputePipelineState, MTLDevice, MTLLibrary};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

type Pipeline = Retained<ProtocolObject<dyn MTLComputePipelineState>>;

/// Metal objects are thread-safe; objc2 just does not mark them so.
struct Cached(Pipeline);
unsafe impl Send for Cached {}

/// Loads the kernel from the embedded metallib, once per kernel. Metal reads
/// libraries from files, so the blob is written to the temp directory.
pub fn pipeline_for(state: &DeviceState, blob: &[u8], kernel: &str) -> Option<Pipeline> {
    static CACHE: Mutex<Option<HashMap<(u64, String), Cached>>> = Mutex::new(None);
    let mut hasher = DefaultHasher::new();
    blob.hash(&mut hasher);
    let key = (hasher.finish(), kernel.to_string());
    let mut cache = CACHE.lock().ok()?;
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(Cached(pipeline)) = cache.get(&key) {
        return Some(pipeline.clone());
    }
    let path = std::env::temp_dir().join(format!("roop-{:016x}.metallib", key.0));
    if !path.exists() {
        std::fs::write(&path, blob).ok()?;
    }
    let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str()?));
    let library = state.device.newLibraryWithURL_error(&url).ok()?;
    let function = library.newFunctionWithName(&NSString::from_str(kernel))?;
    let pipeline = state
        .device
        .newComputePipelineStateWithFunction_error(&function)
        .ok()?;
    cache.insert(key, Cached(pipeline.clone()));
    Some(pipeline)
}
