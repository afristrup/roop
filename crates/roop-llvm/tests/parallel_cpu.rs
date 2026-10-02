mod harness;
mod support;

use harness::{N, harness};

fn roundtrip(options: &roop_llvm::Options) -> String {
    let src = format!(
        "rev fn axpy(a: &mut [i64; {N}], b: &[i64; {N}], i: &mut i64, k: &i64) {{
            #[parallel] from i == 0 {{ a[i] += b[i] * k; }} loop {{ i += 1; }} until i == {};
        }}",
        N - 1
    );
    let ir = format!("{}\n{}", support::ir_with(&src, options), harness());
    support::verify(&ir);
    if let Some(code) = support::run_native(&ir) {
        assert_eq!(code, Some(0), "{ir}");
    }
    ir
}

#[test]
fn parallel_loop_runs_and_reverses_across_threads() {
    roundtrip(&roop_llvm::Options::default());
}

#[test]
fn apple_m4_target_is_applied_to_every_function() {
    let ir = roundtrip(&roop_llvm::Options {
        triple: Some("arm64-apple-macosx".into()),
        cpu: Some("apple-m4".into()),
        ..Default::default()
    });
    assert!(ir.contains("\"target-cpu\"=\"apple-m4\""));
    assert!(ir.contains("define internal void @axpy.par"));
    assert_eq!(
        ir.matches("define ").count(),
        ir.matches(" #0 {").count() + 1
    );
}

#[test]
fn parallel_loop_with_a_failing_assertion_traps() {
    let src = "rev fn f(a: &mut [i64; 4], i: &mut i64) {
        #[parallel] from i == 1 { a[i] += 1; } loop { i += 1; } until i == 3;
    }";
    let ir = format!(
        "{}\ndefine i32 @main() {{ %a = alloca [4 x i64]\n %i = alloca i64\n store i64 0, ptr %i\n call void @f(ptr %a, ptr %i)\n ret i32 0 }}",
        support::ir(src)
    );
    support::verify(&ir);
    if let Some(code) = support::run_native(&ir) {
        assert_ne!(code, Some(0));
    }
}
