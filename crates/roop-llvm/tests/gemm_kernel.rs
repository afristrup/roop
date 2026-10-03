mod support;

use roop_llvm::Options;

fn nest(update: &str) -> String {
    format!(
        "fn mm(c: &mut [[f64; 5]; 4], a: &[[f64; 3]; 4], b: &[[f64; 5]; 3], x: &f64) {{
            ancilla i: i64 = 0;
            #[parallel]
            from i == 0 {{
                ancilla l: i64 = 0;
                from l == 0 {{
                    ancilla j: i64 = 0;
                    from j == 0 {{ {update} }} loop {{ j += 1; }} until j == 4;
                    j -= 4;
                }} loop {{ l += 1; }} until l == 2;
                l -= 2;
            }} loop {{ i += 1; }} until i == 3;
            i -= 3;
        }}"
    )
}

const PRODUCT: &str = "c[i][j] += x * a[i][l] * b[l][j];";

fn with_matrix_unit() -> Options {
    Options {
        sme: true,
        ..Default::default()
    }
}

#[test]
fn a_matrix_product_nest_calls_the_kernel_forward_and_negated_backward() {
    let ir = support::ir_with(&nest(PRODUCT), &with_matrix_unit());
    support::verify(&ir);
    assert_eq!(ir.matches("call void @roop_dgemm(").count(), 2, "{ir}");
    assert_eq!(ir.matches("fneg double").count(), 1, "{ir}");
    assert!(ir.contains("i64 4, i64 5, i64 3)"), "{ir}");
}

#[test]
fn without_the_matrix_unit_the_loops_stay() {
    let ir = support::ir_with(&nest(PRODUCT), &Options::default());
    assert!(!ir.contains("call void @roop_dgemm("), "{ir}");
}

#[test]
fn a_nest_that_is_not_a_matrix_product_stays_loops() {
    for update in [
        "c[i][j] += x * a[i][l] * b[l][j] * x;",
        "c[i][j] -= x * a[i][l] * b[l][j];",
        "c[j][i] += x * a[i][l] * b[l][j];",
    ] {
        let program = support::ir_with(&nest(update), &with_matrix_unit());
        assert!(!program.contains("call void @roop_dgemm("), "{update}");
    }
}
