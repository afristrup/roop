mod support;

use std::process::Command;
use support::{project, roop, stderr};

const ROOP: &str = env!("CARGO_BIN_EXE_roop");

/// A chain of `links` dependent ancilla updates per element: a lot of
/// arithmetic per byte that the compiler cannot fold away.
fn chain_loop(links: usize) -> String {
    let opens: String = (1..=links)
        .map(|j| format!("ancilla t{j}: i64 = 0 {{ "))
        .collect();
    let forward: Vec<String> = std::iter::once("t1 += b[i] * k;".to_string())
        .chain((2..=links).map(|j| format!("t{j} += t{p} * t{p} + b[i];", p = j - 1)))
        .collect();
    let backward: Vec<String> = (2..=links)
        .rev()
        .map(|j| format!("t{j} -= t{p} * t{p} + b[i];", p = j - 1))
        .chain(std::iter::once("t1 -= b[i] * k;".to_string()))
        .collect();
    format!(
        "fn heavy(a: &mut [i64; 1000000], b: &[i64; 1000000], i: &mut i64, k: &i64) {{
            #[parallel] from i == 0 {{ {opens}{} a[i] += t{links}; {} {} }} loop {{ i += 1; }} until i == 999999;
        }}",
        forward.join(" "),
        backward.join(" "),
        "}".repeat(links)
    )
}

/// One multiply-add per element: bound by memory, not arithmetic.
fn streaming_loop() -> &'static str {
    "fn axpy(a: &mut [i64; 4000000], b: &[i64; 4000000], i: &mut i64, k: &i64) {
        #[parallel] from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == 3999999;
    }"
}

#[test]
fn arithmetic_heavy_loop_goes_to_the_gpu_unless_auto_is_off() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let on = project("auto-on", "[parallel]\nauto = true\n", &chain_loop(100));
    assert!(
        roop(&on, &["build", "prog.roop", "--emit", "ir"])
            .status
            .success()
    );
    let ir = std::fs::read_to_string(on.join("prog.ll")).unwrap();
    assert!(ir.contains("call i32 @roop_gpu_dispatch(i32 0"), "{ir}");
    assert!(on.join("prog.air.ll").exists());

    let off = project("auto-off", "[parallel]\nauto = false\n", &chain_loop(100));
    assert!(
        roop(&off, &["build", "prog.roop", "--emit", "ir"])
            .status
            .success()
    );
    let ir = std::fs::read_to_string(off.join("prog.ll")).unwrap();
    assert!(ir.contains("call void @roop_parallel_for"));
    assert!(!ir.contains("call i32 @roop_gpu_dispatch"));
    assert!(!off.join("prog.air.ll").exists());
}

#[test]
fn memory_bound_loop_stays_on_the_cpu_even_with_auto_on() {
    let dir = project("streaming", "", streaming_loop());
    assert!(
        roop(&dir, &["build", "prog.roop", "--emit", "ir"])
            .status
            .success()
    );
    let ir = std::fs::read_to_string(dir.join("prog.ll")).unwrap();
    assert!(ir.contains("call void @roop_parallel_for"));
    assert!(!ir.contains("call i32 @roop_gpu_dispatch"));
}

#[test]
fn dropping_a_target_from_roop_toml_opts_out_of_it() {
    let src = "fn f(a: &mut [i64; 8], i: &mut i64) {
        #[parallel(metal)] from i == 0 { a[i] += 1; } loop { i += 1; } until i == 7;
    }";
    let dir = project("opt-out", "[parallel]\ntargets = [\"cpu\"]\n", src);
    let out = roop(&dir, &["build", "prog.roop", "--emit", "ir"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("disabled in Roop.toml"),
        "{}",
        stderr(&out)
    );

    let allowed = project("opt-in", "", src);
    assert!(
        roop(&allowed, &["build", "prog.roop", "--emit", "ir"])
            .status
            .success()
    );
}

#[test]
fn reports_checker_and_config_errors() {
    let racy = "fn f(a: &mut [i64; 8], s: &mut i64, i: &mut i64) {
        #[parallel] from i == 0 { s += a[i]; } loop { i += 1; } until i == 7;
    }";
    let dir = project("racy", "", racy);
    let out = roop(&dir, &["build", "prog.roop", "--emit", "ir"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("iterations collide"),
        "{}",
        stderr(&out)
    );

    let bad = project("bad-config", "[parallel]\ntargets = [\"tpu\"]\n", racy);
    let out = roop(&bad, &["build", "prog.roop", "--emit", "ir"]);
    assert!(
        stderr(&out).contains("unknown parallel target `tpu`"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn builds_and_runs_a_linked_program_with_fused_loops() {
    let src =
        "fn two(a: &mut [i64; 1000], b: &mut [i64; 1000], i: &mut i64, j: &mut i64, k: &i64) {
        #[parallel] from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == 999;
        #[parallel] from j == 0 { b[j] += a[j]; } loop { j += 1; } until j == 999;
    }";
    let dir = project("run", "", src);
    std::fs::write(
        dir.join("main.c"),
        r#"
#include <stdint.h>
void two(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void two_inv(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
static int64_t a[1000], b[1000];
int main(void) {
    int64_t i = 0, j = 0, k = 3;
    for (int64_t m = 0; m < 1000; m++) { a[m] = m; b[m] = 2 * m; }
    two(a, b, &i, &j, &k);
    for (int64_t m = 0; m < 1000; m++) if (a[m] != 7 * m || b[m] != 9 * m) return 1;
    two_inv(a, b, &i, &j, &k);
    for (int64_t m = 0; m < 1000; m++) if (a[m] != m || b[m] != 2 * m) return 2;
    return 0;
}
"#,
    )
    .unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        Command::new(dir.join("prog")).status().unwrap().code(),
        Some(0)
    );
}

#[test]
fn prints_usage_for_bad_arguments() {
    let out = Command::new(ROOP).arg("compile").output().unwrap();
    assert!(!out.status.success());
    assert!(stderr(&out).contains("usage: roop build"));
}

#[test]
fn lean_subcommand_writes_a_model_and_reports_what_it_proved() {
    let src = "fn add(x: &mut i64, k: &i64) { x += k; }
               fn count(x: &mut i64, i: &mut i64, n: &i64) {
                   from i == 0 { x += 2; } loop { i += 1; } until i == n;
               }
               fn scale(a: &mut [i64; 4], b: &[i64; 4], i: &mut i64) {
                   #[parallel] from i == 0 { a[i] += b[i]; } loop { i += 1; } until i == 3;
               }
               irrev fn wipe(x: &mut i64) { x = 0; }
               fn pipe(x: &mut i64, y: &mut i64) {
                   chan c: i64 {
                       #[concurrent] { send c <- x; }
                       #[concurrent] { recv c -> y; }
                   }
               }";
    let dir = project("lean", "", src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        report.contains("proved by Lean: add, count, scale"),
        "{report}"
    );
    assert!(report.contains("order-independent: scale"), "{report}");
    assert!(!report.contains("open"), "{report}");
    assert!(report.contains("forward model only: wipe"), "{report}");
    assert!(report.contains("skipped pipe"), "{report}");
    assert!(report.contains("Lean accepted the file"), "{report}");
    let lean = std::fs::read_to_string(dir.join("prog.lean")).unwrap();
    assert!(lean.contains("theorem Thm.\u{ab}add_inv_f\u{bb}"));
}

#[test]
fn lean_subcommand_reports_checker_errors() {
    let dir = project("lean-bad", "", "fn f(x: &mut i64) { x = 1; }");
    let out = roop(&dir, &["lean", "prog.roop"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("irrev"), "{}", stderr(&out));
}

#[test]
fn builds_and_proves_a_program_that_imports_from_a_roop_toml_module() {
    let dir = project(
        "modules",
        "[modules]\nlib = \"lib\"\n",
        "use lib::add;\nfn twice(x: &mut i64, k: &i64) { call add(x, k); call add(x, k); }",
    );
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(
        dir.join("lib/mod.roop"),
        "pub fn add(x: &mut i64, k: &i64) { x += k; }",
    )
    .unwrap();
    let out = roop(&dir, &["build", "prog.roop", "--emit", "ir"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let ir = std::fs::read_to_string(dir.join("prog.ll")).unwrap();
    assert!(ir.contains("@lib__add"), "{ir}");

    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(report.contains("lib__add, twice"), "{report}");
}

#[test]
fn emitted_cuda_contract_records_kernel_buffers() {
    let source = "fn add(q: &[i64; 4], p: &mut [i64; 4], i: &mut i64) {
        #[parallel(cuda)] from i == 0 { p[i] += q[i]; } loop { i += 1; } until i == 3;
    }";
    let dir = project(
        "cuda-contract",
        "[parallel]\ntargets = [\"cuda\"]\n",
        source,
    );
    let output = roop(&dir, &["build", "prog.roop", "--emit", "ir"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(dir.join("prog.cuda-contract.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(manifest["schema"], "roop.cuda-contract.v1");
    let kernels = manifest["kernels"].as_array().unwrap();
    assert_eq!(kernels.len(), 2);
    let ptx = std::fs::read_to_string(dir.join("prog.ptx.ll")).unwrap();
    for kernel in kernels {
        let name = kernel["kernel"].as_str().unwrap();
        let header = format!("define ptx_kernel void @{}", name);
        assert!(ptx.contains(&header), "{ptx}");
    }
    let forward = kernels
        .iter()
        .find(|kernel| kernel["direction"] == "forward")
        .unwrap();
    assert_eq!(forward["induction"], "i");
    let buffers = forward["buffers"].as_array().unwrap();
    assert!(buffers.iter().any(|buffer| {
        buffer["name"] == "q"
            && buffer["role"] == "capture"
            && buffer["bytes"] == 32
            && buffer["writable"] == false
    }));
    assert!(buffers.iter().any(|buffer| {
        buffer["name"] == "p"
            && buffer["role"] == "capture"
            && buffer["bytes"] == 32
            && buffer["writable"] == true
    }));
    assert!(
        buffers
            .iter()
            .any(|buffer| buffer["role"] == "launch_params")
    );
    assert!(buffers.iter().any(|buffer| buffer["role"] == "error"));
}

#[test]
fn emitted_cuda_coupling_roundtrips_machine_integers() {
    let source = "fn add(q: &[i64; 4096], p: &mut [i64; 4096], i: &mut i64) {
        #[parallel(cuda)] from i == 0 { p[i] += q[i]; } loop { i += 1; } until i == 4095;
    }";
    let dir = project(
        "cuda-coupling",
        "[parallel]\ntargets = [\"cuda\"]\n",
        source,
    );
    std::fs::write(
        dir.join("main.c"),
        r#"
#include <stdint.h>
#include <string.h>
#define N 4096
void add(int64_t*, int64_t*, int64_t*);
void add_inv(int64_t*, int64_t*, int64_t*);
int main(void) {
    int64_t q[N], p[N];
    uint64_t q0[N], p0[N];
    for (int n = 0; n < N; n++) {
        q0[n] = UINT64_C(0x8000000000000000) + UINT64_C(0x9e3779b97f4a7c15) * (uint64_t)n;
        p0[n] = UINT64_C(0x7fffffffffffffff) - UINT64_C(0xd1b54a32d192ed03) * (uint64_t)n;
        memcpy(&q[n], &q0[n], sizeof q[n]);
        memcpy(&p[n], &p0[n], sizeof p[n]);
    }
    int64_t i = 0;
    add(q, p, &i);
    for (int n = 0; n < N; n++) {
        if ((uint64_t)p[n] != p0[n] + q0[n]) return n + 1;
        if ((uint64_t)q[n] != q0[n]) return n + N + 1;
    }
    add_inv(q, p, &i);
    for (int n = 0; n < N; n++) if ((uint64_t)p[n] != p0[n]) return n + 2 * N + 1;
    return i != 0;
}
"#,
    )
    .unwrap();
    let lean = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(lean.status.success(), "{}", stderr(&lean));
    let output = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        Command::new(dir.join("prog")).status().unwrap().code(),
        Some(0)
    );
}
