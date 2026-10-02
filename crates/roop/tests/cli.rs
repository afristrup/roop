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
    let status = Command::new(env!("CARGO"))
        .args(["build", "-p", "roop-rt"])
        .status()
        .unwrap();
    assert!(status.success());
    let target = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/../../target", env!("CARGO_MANIFEST_DIR")));
    PathBuf::from(target).join("debug/libroop_rt.a")
}

/// 64 updates per element: plenty of work per byte, so the GPU is estimated faster.
fn heavy_loop() -> String {
    let body = ["a[i] += b[i] * k;"; 64].join(" ");
    format!(
        "rev fn heavy(a: &mut [i64; 1000000], b: &[i64; 1000000], i: &mut i64, k: &i64) {{
            #[parallel] from i == 0 {{ {body} }} loop {{ i += 1; }} until i == 999999;
        }}"
    )
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn heavy_loop_goes_to_the_gpu_unless_auto_is_off() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let on = project("auto-on", "[parallel]\nauto = true\n", &heavy_loop());
    assert!(
        roop(&on, &["build", "prog.roop", "--emit", "ir"])
            .status
            .success()
    );
    let ir = std::fs::read_to_string(on.join("prog.ll")).unwrap();
    assert!(ir.contains("call i32 @roop_gpu_dispatch(i32 0"), "{ir}");
    assert!(on.join("prog.air.ll").exists());

    let off = project("auto-off", "[parallel]\nauto = false\n", &heavy_loop());
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
fn dropping_a_target_from_roop_toml_opts_out_of_it() {
    let src = "rev fn f(a: &mut [i64; 8], i: &mut i64) {
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
    let racy = "rev fn f(a: &mut [i64; 8], s: &mut i64, i: &mut i64) {
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
        "rev fn two(a: &mut [i64; 1000], b: &mut [i64; 1000], i: &mut i64, j: &mut i64, k: &i64) {
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
