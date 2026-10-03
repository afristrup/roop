mod support;

use roop_llvm::Options;

fn loop_over(len: usize, update: &str) -> String {
    format!(
        "fn axpy(y: &mut [f64; {len}], x: &[f64; {len}], k: &f64) {{
            ancilla i: i64 = 0;
            #[parallel]
            from i == 0 {{ {update} }} loop {{ i += 1; }} until i == {};
            i -= {};
        }}",
        len - 1,
        len - 1
    )
}

const UPDATE: &str = "y[i] += k * x[i];";

fn with_matrix_unit() -> Options {
    Options {
        sme: true,
        ..Default::default()
    }
}

#[test]
fn a_long_axpy_loop_calls_the_kernel_forward_and_negated_backward() {
    let ir = support::ir_with(&loop_over(4096, UPDATE), &with_matrix_unit());
    support::verify(&ir);
    assert_eq!(ir.matches("call void @roop_daxpy(").count(), 2, "{ir}");
    assert_eq!(ir.matches("fneg double").count(), 1, "{ir}");
    assert!(ir.contains("i64 4096)"), "{ir}");
}

#[test]
fn a_short_loop_stays_a_loop() {
    let ir = support::ir_with(&loop_over(64, UPDATE), &with_matrix_unit());
    assert!(!ir.contains("call void @roop_daxpy("), "{ir}");
}

#[test]
fn without_the_matrix_unit_the_loop_stays() {
    let ir = support::ir_with(&loop_over(4096, UPDATE), &Options::default());
    assert!(!ir.contains("call void @roop_daxpy("), "{ir}");
}

#[test]
fn a_loop_that_is_not_an_axpy_stays_a_loop() {
    for update in [
        "y[i] += k * x[i] * k;",
        "y[i] -= k * x[i];",
        "y[i] += x[i] * k;",
    ] {
        let ir = support::ir_with(&loop_over(4096, update), &with_matrix_unit());
        assert!(!ir.contains("call void @roop_daxpy("), "{update}");
    }
}
