/// Estimates used to decide whether a parallel loop runs faster on CPU
/// threads or on a GPU. Rates are in operations (see `BodyFeatures::work`) or
/// bytes per nanosecond. The defaults are measurements from an Apple M4
/// (10 cores, release runtime, `-O2`): thread start-up about 85 us, Metal
/// launch about 260 us, about 11 GB/s effective copying to the GPU, and about
/// 25 operations per ns across all cores on a memory-bound loop. The GPU
/// compute rate is conservative; arrays are copied in and out, so only loops
/// with a lot of work per byte reach the GPU.
#[derive(Clone, Debug, PartialEq)]
pub struct CostModel {
    pub cpu_threads: u64,
    pub cpu_launch_ns: f64,
    pub cpu_ops_per_ns_per_thread: f64,
    pub gpu_launch_ns: f64,
    pub gpu_ops_per_ns: f64,
    pub copy_bytes_per_ns: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        CostModel {
            cpu_threads: 10,
            cpu_launch_ns: 85_000.0,
            cpu_ops_per_ns_per_thread: 2.5,
            gpu_launch_ns: 260_000.0,
            gpu_ops_per_ns: 500.0,
            copy_bytes_per_ns: 11.0,
        }
    }
}
