use crate::gpu::{RoopBuf, UNAVAILABLE};

#[cfg(not(target_os = "macos"))]
pub fn metal_dispatch(_: &[u8], _: &str, _: &[RoopBuf], _: [i64; 3]) -> i32 {
    UNAVAILABLE
}

/// Shares the buffers with the GPU, runs one thread per iteration, and brings
/// the written buffers back. `params` is `[lo, step, count]`.
#[cfg(target_os = "macos")]
pub fn metal_dispatch(blob: &[u8], kernel: &str, bufs: &[RoopBuf], params: [i64; 3]) -> i32 {
    use crate::gpu::metal::{Staged, device_state, finish_buffer, pipeline_for, stage_buffer};
    use crate::gpu::{KERNEL_ASSERTION_FAILED, OK};
    use objc2_metal::{
        MTLBuffer, MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandEncoder, MTLCommandQueue,
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

    let mut params_bytes: Vec<u8> = params.iter().flat_map(|p| p.to_ne_bytes()).collect();
    let mut error_bytes = 0i32.to_ne_bytes();
    let mut staged: Vec<Staged> = Vec::new();
    let mut hosts: Vec<*mut u8> = bufs.iter().map(|b| b.ptr.cast()).collect();
    hosts.push(params_bytes.as_mut_ptr());
    hosts.push(error_bytes.as_mut_ptr());
    let sizes = bufs
        .iter()
        .map(|b| b.size as usize)
        .chain([params_bytes.len(), error_bytes.len()]);
    for (host, size) in hosts.iter().zip(sizes) {
        match unsafe { stage_buffer(state, *host, size) } {
            Some(buffer) => staged.push(buffer),
            None => return UNAVAILABLE,
        }
    }
    for (index, buffer) in staged.iter().enumerate() {
        unsafe { encoder.setBuffer_offset_atIndex(Some(&buffer.buffer), 0, index) };
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
    let completed = command.status() == MTLCommandBufferStatus::Completed;

    let error = unsafe { *(staged[bufs.len() + 1].buffer.contents().as_ptr() as *const i32) };
    let mut writes: Vec<bool> = bufs.iter().map(|b| b.written != 0).collect();
    writes.extend([false, false]);
    for ((buffer, host), written) in staged.into_iter().zip(hosts).zip(writes) {
        unsafe { finish_buffer(buffer, host, written && completed && error == 0) };
    }
    match (completed, error) {
        (false, _) => UNAVAILABLE,
        (true, 0) => OK,
        (true, _) => KERNEL_ASSERTION_FAILED,
    }
}
