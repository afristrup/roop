use crate::{Choice, CostModel};
use roop_syntax::Target;

/// Compares the estimated times. The GPU pays a fixed cost (launch plus
/// copying the buffers over) and then runs iterations faster; it wins once
/// enough iterations amortize that cost.
pub fn choose_target(
    model: &CostModel,
    work: u64,
    bytes: u64,
    trip: Option<i64>,
    gpu: Target,
) -> Choice {
    let work = work.max(1) as f64;
    let cpu_per_iter = work / (model.cpu_threads as f64 * model.cpu_ops_per_ns_per_thread);
    let gpu_per_iter = work / model.gpu_ops_per_ns;
    if gpu_per_iter >= cpu_per_iter {
        return Choice::Cpu;
    }
    let fixed = model.gpu_launch_ns + bytes as f64 / model.copy_bytes_per_ns;
    let break_even = fixed / (cpu_per_iter - gpu_per_iter);
    match trip {
        Some(n) if (n as f64) > break_even => Choice::Gpu(gpu),
        Some(_) => Choice::Cpu,
        None => Choice::Runtime {
            gpu,
            break_even: break_even.ceil() as i64,
        },
    }
}
