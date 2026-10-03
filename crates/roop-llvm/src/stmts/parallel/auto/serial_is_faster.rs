use crate::CostModel;
use roop_check::BodyFeatures;

/// Whether the whole loop, run on one thread, costs less than the cutoff,
/// which is about the price of launching threads. Then threads cannot win.
pub fn serial_is_faster(model: &CostModel, features: &BodyFeatures, trip: i64) -> bool {
    let work = features.work.max(1) as f64;
    let moved = features.bytes as f64;
    let per_iteration =
        (work / model.cpu_ops_per_ns_per_thread).max(moved / model.cpu_bytes_per_ns);
    trip as f64 * per_iteration <= model.serial_cutoff_ns
}
