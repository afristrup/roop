mod support;
mod two;

use roop_llvm::{CostModel, Options, ParallelOptions};
use roop_syntax::Target;
use two::{HARNESS, program};

/// The two-loop program with a bare `#[parallel]` on both loops.
fn bare() -> String {
    program("cpu").replace("#[parallel(cpu)]", "#[parallel]")
}

fn metal_available() -> bool {
    cfg!(target_os = "macos")
        && std::process::Command::new("xcrun")
            .args(["-sdk", "macosx", "--find", "metal"])
            .output()
            .is_ok_and(|o| o.status.success())
}

fn options(gpus: &[Target], cost: CostModel) -> Options {
    Options {
        parallel: ParallelOptions {
            auto_gpus: gpus.to_vec(),
            cost,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The GPU has no fixed cost at all, so it wins at any size.
fn gpu_always() -> CostModel {
    CostModel {
        cpu_launch_ns: 1000.0,
        serial_cutoff_ns: 0.0,
        gpu_launch_ns: 0.0,
        copy_bytes_per_ns: 1e9,
        gpu_ops_per_ns: 1e6,
        gpu_bytes_per_ns: 1e9,
        ..CostModel::default()
    }
}

/// The GPU is much faster per iteration but costs a little to launch, so it
/// wins only from a few thousand iterations up.
fn gpu_when_large() -> CostModel {
    CostModel {
        cpu_launch_ns: 0.0,
        serial_cutoff_ns: 0.0,
        gpu_launch_ns: 1000.0,
        copy_bytes_per_ns: 1e9,
        gpu_ops_per_ns: 1e6,
        gpu_bytes_per_ns: 1e9,
        ..CostModel::default()
    }
}

/// Launching a kernel costs a full second.
fn gpu_hostile() -> CostModel {
    CostModel {
        gpu_launch_ns: 1e9,
        serial_cutoff_ns: 0.0,
        ..CostModel::default()
    }
}

#[test]
fn bare_parallel_stays_on_the_cpu_without_gpu_candidates() {
    let out = support::compiled(&bare(), &options(&[], gpu_always()));
    assert!(out.host.contains("@roop_parallel_for("));
    assert!(!out.host.contains("call i32 @roop_gpu_dispatch"));
    assert!(out.air.is_none());
}

#[test]
fn estimated_cost_decides_between_cpu_and_gpu() {
    let src = bare();
    let gpu = support::compiled(&src, &options(&[Target::Metal], gpu_always()));
    assert!(gpu.air.is_some());
    assert!(gpu.host.contains("call i32 @roop_gpu_dispatch(i32 0"));
    assert!(!gpu.host.contains("call void @roop_parallel_for"));
    let cpu = support::compiled(&src, &options(&[Target::Metal], gpu_hostile()));
    assert!(cpu.air.is_none());
    assert!(cpu.host.contains("call void @roop_parallel_for"));
}

#[test]
fn a_literal_trip_count_below_break_even_stays_on_the_cpu() {
    let out = support::compiled(&bare(), &options(&[Target::Metal], gpu_when_large()));
    assert!(out.air.is_none());
    assert!(out.host.contains("call void @roop_parallel_for"));
}

#[test]
fn unknown_trip_count_branches_at_run_time() {
    let src = "fn f(a: &mut [i64; 8], i: &mut i64, n: &i64) {
        #[parallel] from i == 0 { a[i] += 1; } loop { i += 1; } until i == n;
    }";
    let out = support::compiled(src, &options(&[Target::Metal], gpu_when_large()));
    assert!(out.air.is_some());
    assert!(out.host.contains("call void @roop_parallel_for"));
    assert!(out.host.contains("call i32 @roop_gpu_dispatch"));
    assert!(out.host.contains("icmp sge i64"));
}

#[test]
fn double_precision_loops_never_go_to_the_apple_gpu() {
    let src = "fn f(a: &mut [f64; 8], i: &mut i64) {
        #[parallel] from i == 0 { a[i] += 1.0; } loop { i += 1; } until i == 7;
    }";
    let out = support::compiled(src, &options(&[Target::Metal], gpu_always()));
    assert!(out.air.is_none());
    let cuda = support::compiled(src, &options(&[Target::Nvptx], gpu_always()));
    assert!(cuda.ptx.is_some());
}

#[test]
fn loops_with_calls_stay_on_the_cpu() {
    let src = "fn g(x: &mut i64) { x += 1; }
        fn f(a: &mut [i64; 8], i: &mut i64) {
            #[parallel] from i == 0 { call g(a[i]); } loop { i += 1; } until i == 7;
        }";
    let out = support::compiled(src, &options(&[Target::Metal, Target::Nvptx], gpu_always()));
    assert!(out.air.is_none() && out.ptx.is_none());
}

#[test]
fn naming_a_disabled_target_is_an_error() {
    let src = "fn f(a: &mut [i64; 8], i: &mut i64) {
        #[parallel(metal)] from i == 0 { a[i] += 1; } loop { i += 1; } until i == 7;
    }";
    let program = roop_syntax::parse(src).unwrap();
    let opts = Options {
        parallel: ParallelOptions {
            allowed: vec![Target::Cpu],
            ..Default::default()
        },
        ..Default::default()
    };
    let err = roop_llvm::compile_all(&program, &opts).err().unwrap();
    assert!(err.to_string().contains("disabled in Roop.toml"));
}

#[test]
fn an_automatically_chosen_gpu_produces_the_same_results() {
    if !metal_available() {
        return;
    }
    let src = bare();
    let out = support::compiled(&src, &options(&[Target::Metal], gpu_always()));
    let lib = support::metallib(out.air.as_ref().unwrap()).unwrap();
    let host_ir = roop_llvm::embed_blobs(&out.host, Some(&lib), None);
    let files = [
        ("m0.ll".to_string(), host_ir.as_str()),
        ("main.c".to_string(), HARNESS),
    ];
    assert_eq!(support::run_native_files(&files).unwrap(), Some(0));
}

#[test]
fn a_loop_too_small_to_pay_for_threads_runs_serially_by_default() {
    let src = "fn f(a: &mut [i64; 8], i: &mut i64) {
        #[parallel] from i == 0 { a[i] += 1; } loop { i += 1; } until i == 7;
    }";
    let out = support::compiled(src, &Options::default());
    assert!(
        !out.host.contains("call void @roop_parallel_for"),
        "{}",
        out.host
    );
    let forced = Options {
        parallel: ParallelOptions {
            cost: CostModel {
                serial_cutoff_ns: 0.0,
                ..CostModel::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let out = support::compiled(src, &forced);
    assert!(out.host.contains("call void @roop_parallel_for"));
}
