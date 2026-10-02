mod harness;
mod support;

use harness::{N, harness};
use roop_llvm::Options;

fn metal_available() -> bool {
    cfg!(target_os = "macos")
        && std::process::Command::new("xcrun")
            .args(["-sdk", "macosx", "--find", "metal"])
            .output()
            .is_ok_and(|o| o.status.success())
}

fn program(target: &str) -> String {
    format!(
        "rev fn axpy(a: &mut [i64; {N}], b: &[i64; {N}], i: &mut i64, k: &i64) {{
            #[parallel{target}] from i == 0 {{ a[i] += b[i] * k; }} loop {{ i += 1; }} until i == {};
        }}",
        N - 1
    )
}

#[test]
fn metal_kernel_is_emitted_as_air_with_buffer_metadata() {
    let out = support::compiled(&program("(metal)"), &Options::default());
    let air = out.air.expect("an AIR module");
    assert!(air.contains("!air.kernel"));
    assert!(air.contains("\"air.read_write\""));
    assert!(air.contains("\"air.thread_position_in_grid\""));
    assert!(out.host.contains("call i32 @roop_gpu_dispatch(i32 0"));
    assert!(out.ptx.is_none());
    if metal_available() {
        support::metallib(&air).expect("Apple's compiler accepts the AIR");
    }
}

#[test]
fn parallel_loop_runs_and_reverses_on_the_apple_gpu() {
    if !metal_available() {
        return;
    }
    let out = support::compiled(&program("(metal)"), &Options::default());
    let lib = support::metallib(out.air.as_ref().unwrap()).unwrap();
    let host_ir = roop_llvm::embed_blobs(&out.host, Some(&lib), None);
    let host = format!("{}\n{}", host_ir, harness());
    support::verify(&host);
    let code = support::run_native_modules(&[&host, &blob]).unwrap();
    assert_eq!(code, Some(0));
}
