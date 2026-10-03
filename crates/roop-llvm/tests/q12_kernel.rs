mod support;

use roop_llvm::Options;

fn nest(c: &str, a: &str, b: &str, update: &str) -> String {
    format!(
        "fn mm(c: &mut {c}, a: &{a}, b: &{b}) {{
            ancilla i: i64 = 0;
            #[parallel]
            from i == 0 {{
                ancilla k: i64 = 0;
                from k == 0 {{
                    ancilla j: i64 = 0;
                    from j == 0 {{ {update} }} loop {{ j += 1; }} until j == 2;
                    j -= 2;
                }} loop {{ k += 1; }} until k == 4;
                k -= 4;
            }} loop {{ i += 1; }} until i == 3;
            i -= 3;
        }}"
    )
}

const MATMUL: &str = "c[i][k] += a[i][j] * b[j][k] / 4096;";
const NT: &str = "c[i][k] += a[i][j] * b[k][j] / 4096;";
const TN: &str = "c[i][k] += a[j][i] * b[j][k] / 4096;";

fn with_kernel() -> Options {
    Options {
        q12: true,
        ..Default::default()
    }
}

fn plain_nn(update: &str) -> String {
    nest("[[i64; 5]; 4]", "[[i64; 3]; 4]", "[[i64; 5]; 3]", update)
}

#[test]
fn a_q12_product_nest_calls_the_kernel_forward_and_negated_backward() {
    let ir = support::ir_with(&plain_nn(MATMUL), &with_kernel());
    support::verify(&ir);
    assert_eq!(ir.matches("call void @roop_q12_matmul(").count(), 2, "{ir}");
    assert!(ir.contains("i64 1, i64 4, i64 5, i64 3, i64 0)"), "{ir}");
    assert!(ir.contains("i64 -1, i64 4, i64 5, i64 3, i64 0)"), "{ir}");
}

#[test]
fn the_transposed_forms_pass_their_layout() {
    let nt = nest("[[i64; 5]; 4]", "[[i64; 3]; 4]", "[[i64; 3]; 5]", NT);
    let tn = nest("[[i64; 5]; 4]", "[[i64; 4]; 3]", "[[i64; 5]; 3]", TN);
    for (program, layout) in [(nt, 1), (tn, 2)] {
        let ir = support::ir_with(&program, &with_kernel());
        support::verify(&ir);
        assert!(ir.contains(&format!("i64 3, i64 {layout})")), "{ir}");
    }
}

#[test]
fn without_the_kernel_the_loops_stay() {
    let ir = support::ir_with(&plain_nn(MATMUL), &Options::default());
    assert!(!ir.contains("call void @roop_q12_matmul("), "{ir}");
}

#[test]
fn a_nest_that_is_not_a_q12_product_stays_loops() {
    for update in [
        "c[i][k] += a[i][j] * b[j][k];",
        "c[i][k] += a[i][j] * b[j][k] / 2048;",
        "c[i][k] -= a[i][j] * b[j][k] / 4096;",
        "c[i][k] += a[i][j] * a[i][j] / 4096;",
        "c[k][i] += a[i][j] * b[j][k] / 4096;",
    ] {
        let ir = support::ir_with(&plain_nn(update), &with_kernel());
        assert!(!ir.contains("call void @roop_q12_matmul("), "{update}");
    }
}

#[test]
fn matrices_of_the_wrong_shape_stay_loops() {
    let program = nest("[[i64; 5]; 4]", "[[i64; 3]; 4]", "[[i64; 4]; 3]", MATMUL);
    assert!(!support::ir_with(&program, &with_kernel()).contains("call void @roop_q12_matmul("));
}
