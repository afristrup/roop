/// Estimates used to decide whether a parallel loop runs faster on CPU
/// threads or on a GPU. Rates are in operations (see `BodyFeatures::work`) or
/// bytes per nanosecond; the defaults were measured on an Apple M4.
#[derive(Clone, Debug, PartialEq)]
pub struct CostModel {
    pub cpu_threads: u64,
    pub cpu_ops_per_ns_per_thread: f64,
    pub gpu_launch_ns: f64,
    pub gpu_ops_per_ns: f64,
    pub copy_bytes_per_ns: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        CostModel {
            cpu_threads: 10,
            cpu_ops_per_ns_per_thread: 1.0,
            gpu_launch_ns: 200_000.0,
            gpu_ops_per_ns: 50.0,
            copy_bytes_per_ns: 20.0,
        }
    }
}
