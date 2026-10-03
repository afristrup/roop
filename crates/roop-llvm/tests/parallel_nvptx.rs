mod support;

use roop_llvm::{Options, compile_all};
use roop_syntax::parse;
use std::io::Write;
use std::process::{Command, Stdio};

fn lower_to_ptx(ir: &str) -> Option<String> {
    let llc = ["llc", "/opt/homebrew/opt/llvm/bin/llc"]
        .into_iter()
        .find(|t| {
            Command::new(t)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
        })?;
    let mut child = Command::new(llc)
        .args(["-mtriple=nvptx64-nvidia-cuda", "-mcpu=sm_80", "-o", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(ir.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    // Hosts built without the NVPTX backend cannot lower; skip rather than fail.
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("No available targets") || stderr.contains("unknown target triple") {
        return None;
    }
    assert!(out.status.success(), "{stderr}\n{ir}");
    Some(String::from_utf8(out.stdout).unwrap())
}

#[test]
fn nvptx_loop_lowers_to_a_ptx_entry_point() {
    let src = "fn axpy(a: &mut [f64; 64], b: &[f64; 64], i: &mut i64, k: &f64) {
        #[parallel(cuda)] from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == 63;
    }";
    let out = support::compiled(src, &Options::default());
    let device = out.ptx.expect("a PTX module");
    assert!(device.contains("define ptx_kernel void @axpy_k"));
    assert!(out.host.contains("call i32 @roop_gpu_dispatch(i32 1"));
    assert!(out.air.is_none());
    if let Some(ptx) = lower_to_ptx(&device) {
        assert!(ptx.contains(".visible .entry axpy_k"));
        assert!(ptx.contains("%ctaid.x"));
    }
}

#[test]
fn apple_gpu_rejects_double_precision_but_cuda_accepts_it() {
    let body = "fn f(a: &mut [f64; 4], i: &mut i64) {
        #[parallel(TARGET)] from i == 0 { a[i] += 1.0; } loop { i += 1; } until i == 3;
    }";
    let metal = parse(&body.replace("TARGET", "metal")).unwrap();
    let err = compile_all(&metal, &Options::default()).err().unwrap();
    assert_eq!(
        err.to_string(),
        "unsupported in code generation: f64 on the Apple GPU"
    );
    let cuda = parse(&body.replace("TARGET", "cuda")).unwrap();
    assert!(compile_all(&cuda, &Options::default()).is_ok());
}

#[test]
fn gpu_kernels_reject_calls_and_use_the_error_flag_for_assertions() {
    let call = "fn g(x: &mut i64) { x += 1; }
        fn f(a: &mut [i64; 4], i: &mut i64) {
            #[parallel(metal)] from i == 0 { call g(a[i]); } loop { i += 1; } until i == 3;
        }";
    let program = parse(call).unwrap();
    assert!(compile_all(&program, &Options::default()).is_err());
    let assertion = "fn f(a: &mut [i64; 4], i: &mut i64) {
        #[parallel(metal)] from i == 0 { if a[i] > 0 { a[i] += 1; } fi a[i] > 0; } loop { i += 1; } until i == 3;
    }";
    let out = support::compiled(assertion, &Options::default());
    assert!(
        out.air
            .unwrap()
            .contains("store i32 1, i32 addrspace(1)* %err")
    );
}
