use crate::{detect_cpu, detect_sme, host_gpus, parse_target};
use roop_config::Config;
use roop_llvm::{CostModel, Options, ParallelOptions};
use roop_syntax::Target;

/// Combines `Roop.toml` with what this machine offers: the CPU model, the
/// permitted targets, and which GPUs the compiler may pick on its own.
pub fn host_options(config: &Config) -> Options {
    let allowed: Vec<Target> = config
        .parallel
        .targets
        .iter()
        .filter_map(|t| parse_target(t))
        .collect();
    let auto_gpus = if config.parallel.auto {
        host_gpus()
            .into_iter()
            .filter(|t| allowed.contains(t))
            .collect()
    } else {
        Vec::new()
    };
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let cpu = config.parallel.cpu.clone().or_else(detect_cpu);
    Options {
        no_entry: false,
        history_limit: config.world.history_limit,
        triple: cfg!(all(target_os = "macos", target_arch = "aarch64"))
            .then(|| "arm64-apple-macosx".into()),
        cpu,
        sme: config.parallel.sme && detect_sme(),
        q12: config.parallel.q12 && cfg!(all(target_os = "macos", target_arch = "aarch64")),
        parallel: ParallelOptions {
            allowed,
            auto_gpus,
            cost: CostModel {
                cpu_threads: threads,
                ..CostModel::default()
            },
        },
    }
}
