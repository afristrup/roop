/// Estimates used to decide whether a parallel loop runs faster on CPU
/// threads or on a GPU.
///
/// A loop iteration takes as long as the slower of its arithmetic (`work`
/// operations) and its memory traffic (`bytes`), on each device; each device
/// also pays a fixed cost per launch, and the GPU pays to copy the arrays.
///
/// The defaults were measured on an Apple M4 (10 cores, release runtime,
/// `-O2`, 64-bit integer loops). The CPU side was checked again against
/// `bench/calibrate_dispatch.py`: threads cost 12 to 17 microseconds to start,
/// one thread streams about 150 bytes a nanosecond out of cache, and threads
/// start to win at 128K to 256K elements of a streaming loop. The CPU is strong because its cores
/// vectorize; the GPU emulates 64-bit integer multiplies and streams memory
/// at about a quarter of the CPU's rate, so it only pays off for loops that
/// are heavy on arithmetic per byte.
#[derive(Clone, Debug, PartialEq)]
pub struct CostModel {
    pub cpu_threads: u64,
    pub cpu_launch_ns: f64,
    pub cpu_ops_per_ns_per_thread: f64,
    pub cpu_bytes_per_ns: f64,
    /// What one thread alone streams, which is what a loop run serially gets.
    pub cpu_thread_bytes_per_ns: f64,
    pub gpu_launch_ns: f64,
    pub gpu_ops_per_ns: f64,
    pub gpu_bytes_per_ns: f64,
    pub copy_bytes_per_ns: f64,
    /// A loop whose whole serial run costs less than this does not start
    /// threads at all.
    pub serial_cutoff_ns: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        CostModel {
            cpu_threads: 10,
            cpu_launch_ns: 15_000.0,
            cpu_ops_per_ns_per_thread: 26.0,
            cpu_bytes_per_ns: 90.0,
            cpu_thread_bytes_per_ns: 150.0,
            gpu_launch_ns: 240_000.0,
            gpu_ops_per_ns: 370.0,
            gpu_bytes_per_ns: 27.0,
            copy_bytes_per_ns: 30.0,
            serial_cutoff_ns: 30_000.0,
        }
    }
}
