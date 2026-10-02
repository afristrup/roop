use crate::gpu::{RoopBuf, UNAVAILABLE};

#[cfg(not(target_os = "macos"))]
pub fn metal_dispatch(_: &[u8], _: &str, _: &[RoopBuf], _: [i64; 3]) -> i32 {
    UNAVAILABLE
}

/// Copies the buffers to the GPU, runs one thread per iteration, and copies
/// the written buffers back. `params` is `[lo, step, count]`.
#[cfg(target_os = "macos")]
pub fn metal_dispatch(blob: &[u8], kernel: &str, bufs: &[RoopBuf], params: [i64; 3]) -> i32 {
    use crate::gpu::metal::{device_state, pipeline_for, shared_buffer};
    use crate::gpu::{KERNEL_ASSERTION_FAILED, OK};
    use objc2_metal::{
        MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandEncoder, MTLCommandQueue,
        MTLComputeCommandEncoder, MTLComputePipelineState, MTLSize,
    };

    let Some(state) = device_state() else {
        return UNAVAILABLE;
    };
    let Some(pipeline) = pipeline_for(state, blob, kernel) else {
        return UNAVAILABLE;
    };
    let Some(command) = state.queue.commandBuffer() else {
        return UNAVAILABLE;
    };
    let Some(encoder) = command.computeCommandEncoder() else {
        return UNAVAILABLE;
    };
    encoder.setComputePipelineState(&pipeline);

    let mut device_bufs = Vec::new();
    for buf in bufs {
        let bytes = unsafe { std::slice::from_raw_parts(buf.ptr as *const u8, buf.size as usize) };
        device_bufs.push(shared_buffer(state, bytes));
    }
    let params_bytes: Vec<u8> = params.iter().flat_map(|p| p.to_ne_bytes()).collect();
    device_bufs.push(shared_buffer(state, &params_bytes));
    device_bufs.push(shared_buffer(state, &0i32.to_ne_bytes()));
    for (index, buffer) in device_bufs.iter().enumerate() {
        unsafe { encoder.setBuffer_offset_atIndex(Some(buffer), 0, index) };
    }

    let width = params[2].max(1) as usize;
    let group = pipeline.maxTotalThreadsPerThreadgroup().min(256).min(width);
    encoder.dispatchThreads_threadsPerThreadgroup(
        MTLSize {
            width,
            height: 1,
            depth: 1,
        },
        MTLSize {
            width: group,
            height: 1,
            depth: 1,
        },
    );
    encoder.endEncoding();
    command.commit();
    command.waitUntilCompleted();
    if command.status() != MTLCommandBufferStatus::Completed {
        return UNAVAILABLE;
    }

    let error = unsafe { *(device_bufs[bufs.len() + 1].contents().as_ptr() as *const i32) };
    if error != 0 {
        return KERNEL_ASSERTION_FAILED;
    }
    for (buf, device) in bufs.iter().zip(&device_bufs) {
        if buf.written != 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    device.contents().as_ptr() as *const u8,
                    buf.ptr as *mut u8,
                    buf.size as usize,
                );
            }
        }
    }
    OK
}
