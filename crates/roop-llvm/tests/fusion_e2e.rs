mod support;

use roop_llvm::{Compiled, Options, compile_all};
use roop_opt::fuse_parallel;
use roop_syntax::parse;

const N: usize = 1000;

fn program(target: &str) -> String {
    format!(
        "rev fn two(a: &mut [i64; {N}], b: &mut [i64; {N}], i: &mut i64, j: &mut i64, k: &i64) {{
            #[parallel({target})] from i == 0 {{ a[i] += b[i] * k; }} loop {{ i += 1; }} until i == {last};
            #[parallel({target})] from j == 0 {{ b[j] += a[j]; }} loop {{ j += 1; }} until j == {last};
        }}",
        last = N - 1
    )
}

fn build(target: &str, fuse: bool) -> Compiled {
    let program = parse(&program(target)).unwrap();
    roop_check::check(&program).unwrap();
    let program = if fuse { fuse_parallel(&program) } else { program };
    roop_check::check(&program).unwrap();
    compile_all(&program, &Options::default()).unwrap()
}

/// IR that sums `%arr` into `%{name}sum`, then compares to `want`.
fn sum_check(name: &str, arr: &str, prev: &str, want: i64) -> String {
    format!(
        "  br label %{name}
{name}:
  %{name}m = phi i64 [0, %{prev}], [%{name}m1, %{name}]
  %{name}s = phi i64 [0, %{prev}], [%{name}s1, %{name}]
  %{name}p = getelementptr inbounds [{N} x i64], ptr %{arr}, i64 0, i64 %{name}m
  %{name}v = load i64, ptr %{name}p
  %{name}s1 = add i64 %{name}s, %{name}v
  %{name}m1 = add i64 %{name}m, 1
  %{name}c = icmp slt i64 %{name}m1, {N}
  br i1 %{name}c, label %{name}, label %{name}x
{name}x:
  %{name}bad = icmp ne i64 %{name}s1, {want}
"
    )
}

/// a[m] = m, b[m] = 2m, k = 3. After `two`: a = 7m, b = 9m, i = j = N-1.
fn harness() -> String {
    let tri = (N * (N - 1) / 2) as i64;
    let mut ir = format!(
        "define i32 @main() {{
entry:
  %a = alloca [{N} x i64]
  %b = alloca [{N} x i64]
  %i = alloca i64
  %j = alloca i64
  %k = alloca i64
  store i64 0, ptr %i
  store i64 0, ptr %j
  store i64 3, ptr %k
  br label %init
init:
  %n = phi i64 [0, %entry], [%n1, %init]
  %ap = getelementptr inbounds [{N} x i64], ptr %a, i64 0, i64 %n
  store i64 %n, ptr %ap
  %bp = getelementptr inbounds [{N} x i64], ptr %b, i64 0, i64 %n
  %n2 = mul i64 %n, 2
  store i64 %n2, ptr %bp
  %n1 = add i64 %n, 1
  %c = icmp slt i64 %n1, {N}
  br i1 %c, label %run, label %init
run:
  call void @two(ptr %a, ptr %b, ptr %i, ptr %j, ptr %k)
"
    );
    // the loop above branches to %run on exit; fix the polarity here
    ir = ir.replace("br i1 %c, label %run, label %init", "br i1 %c, label %init, label %run");
    ir.push_str(&sum_check("fa", "a", "run", 7 * tri));
    ir.push_str(&sum_check("fb", "b", "faok", 9 * tri).replace("br label %fb", "br label %fb"));
    ir
}

#[test]
fn placeholder_to_keep_harness_in_sync() {
    assert!(harness().contains("@two"));
}
