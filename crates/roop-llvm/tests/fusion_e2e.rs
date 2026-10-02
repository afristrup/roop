mod support;
mod two;

use two::HARNESS;

use roop_llvm::{Compiled, Options, compile_all};
use roop_opt::fuse_parallel;
use roop_syntax::parse;

fn build(target: &str, fuse: bool) -> Compiled {
    let program = parse(&two::program(target)).unwrap();
    roop_check::check(&program).unwrap();
    let program = if fuse {
        fuse_parallel(&program)
    } else {
        program
    };
    roop_check::check(&program).unwrap();
    compile_all(&program, &Options::default()).unwrap()
}

fn run_cpu(fuse: bool) {
    let out = build("cpu", fuse);
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("main.c".to_string(), HARNESS),
    ];
    if let Some(code) = support::run_native_files(&files) {
        assert_eq!(code, Some(0), "fuse = {fuse}");
    }
}

fn metal_available() -> bool {
    cfg!(target_os = "macos")
        && std::process::Command::new("xcrun")
            .args(["-sdk", "macosx", "--find", "metal"])
            .output()
            .is_ok_and(|o| o.status.success())
}

fn run_metal(fuse: bool) {
    if !metal_available() {
        return;
    }
    let out = build("metal", fuse);
    let lib = support::metallib(out.air.as_ref().unwrap()).unwrap();
    let host_ir = roop_llvm::embed_blobs(&out.host, Some(&lib), None);
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("m1.ll".to_string(), blob.as_str()),
        ("main.c".to_string(), HARNESS),
    ];
    assert_eq!(
        support::run_native_files(&files).unwrap(),
        Some(0),
        "fuse = {fuse}"
    );
}

#[test]
fn cpu_results_match_with_and_without_fusion() {
    run_cpu(false);
    run_cpu(true);
}

#[test]
fn gpu_results_match_with_and_without_fusion() {
    run_metal(false);
    run_metal(true);
}

#[test]
fn fusion_halves_the_kernel_count() {
    let kernels = |fuse| {
        build("metal", fuse)
            .air
            .unwrap()
            .matches("define void @")
            .count()
    };
    assert_eq!(kernels(false), 4); // two loops, forward and inverse
    assert_eq!(kernels(true), 2);
}
