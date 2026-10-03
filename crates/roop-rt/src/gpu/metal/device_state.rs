#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice};
use std::sync::OnceLock;

pub struct DeviceState {
    pub device: Retained<ProtocolObject<dyn MTLDevice>>,
    pub queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
}

/// Metal objects are thread-safe; objc2 just does not mark them so.
struct Shared(Option<DeviceState>);
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

/// The default GPU and one command queue, created on first use.
pub fn device_state() -> Option<&'static DeviceState> {
    static STATE: OnceLock<Shared> = OnceLock::new();
    STATE
        .get_or_init(|| {
            Shared(MTLCreateSystemDefaultDevice().and_then(|device| {
                let queue = device.newCommandQueue()?;
                Some(DeviceState { device, queue })
            }))
        })
        .0
        .as_ref()
}
