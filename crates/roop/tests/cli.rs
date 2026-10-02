use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ROOP: &str = env!("CARGO_BIN_EXE_roop");

fn project(name: &str, config: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("roop-cli-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Roop.toml"), config).unwrap();
    std::fs::write(dir.join("prog.roop"), source).unwrap();
    dir
}

fn roop(dir: &Path, args: &[&str]) -> Output {
    Command::new(ROOP)
        .current_dir(dir)
        .args(args)
        .env("ROOP_RT_LIB", runtime_lib())
        .output()
        .unwrap()
}

fn runtime_lib() -> PathBuf {
    static BUILD: std::sync::Once = std::sync::Once::new();
    BUILD.call_once(|| {
        let cargo = Command::new(env!("CARGO"));
        let status = {
            let mut c = cargo;
            c.args(["build", "-p", "roop-rt"]).status().unwrap()
        };
        assert!(status.success());
    });
    let target = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/../../target", env!("CARGO_MANIFEST_DIR")));
    PathBuf::from(target).join("debug/libroop_rt.a")
}

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

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
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
    assert!(report.contains("proved by Lean: add, count"), "{report}");
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
