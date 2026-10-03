use crate::{Choice, CostModel};
use roop_check::BodyFeatures;
use roop_syntax::Target;

/// Compares estimated times. Each device pays a fixed cost (CPU thread
/// start-up; GPU launch plus copying `buffer_bytes` over) and then a
/// per-iteration cost, the slower of arithmetic and memory. The GPU wins
/// once enough iterations amortize its extra fixed cost.
pub fn choose_target(
    model: &CostModel,
    features: &BodyFeatures,
    buffer_bytes: u64,
    trip: Option<i64>,
    gpu: Target,
) -> Choice {
    let work = features.work.max(1) as f64;
    let moved = features.bytes as f64;
    let cpu_ops = model.cpu_threads as f64 * model.cpu_ops_per_ns_per_thread;
    let cpu_per_iter = (work / cpu_ops).max(moved / model.cpu_bytes_per_ns);
    let gpu_per_iter = (work / model.gpu_ops_per_ns).max(moved / model.gpu_bytes_per_ns);
    let extra_fixed =
        model.gpu_launch_ns + buffer_bytes as f64 / model.copy_bytes_per_ns - model.cpu_launch_ns;
    if extra_fixed <= 0.0 {
        return Choice::Gpu(gpu);
    }
    if gpu_per_iter >= cpu_per_iter {
        return Choice::Cpu;
    }
    let break_even = extra_fixed / (cpu_per_iter - gpu_per_iter);
    match trip {
        Some(n) if (n as f64) > break_even => Choice::Gpu(gpu),
        Some(_) => Choice::Cpu,
        None => Choice::Runtime {
            gpu,
            break_even: break_even.ceil() as i64,
        },
    }
}
