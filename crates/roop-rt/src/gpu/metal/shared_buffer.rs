#![cfg(target_os = "macos")]

use crate::gpu::metal::DeviceState;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLBuffer, MTLDevice, MTLResourceOptions};
use std::ptr::NonNull;

/// A CPU-and-GPU visible buffer initialised with `bytes`.
pub fn shared_buffer(state: &DeviceState, bytes: &[u8]) -> Retained<ProtocolObject<dyn MTLBuffer>> {
    let pointer = NonNull::new(bytes.as_ptr() as *mut _).expect("buffer bytes");
    unsafe {
        state
            .device
            .newBufferWithBytes_length_options(
                pointer,
                bytes.len(),
                MTLResourceOptions::StorageModeShared,
            )
            .expect("Metal buffer allocation")
    }
}
