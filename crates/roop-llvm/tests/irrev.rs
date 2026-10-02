mod support;

use roop_llvm::Options;

fn run(src: &str, main_c: &str) -> i32 {
    let out = support::compiled(src, &Options::default());
    let main = format!("#include <stdint.h>\n{main_c}");
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("main.c".to_string(), main.as_str()),
    ];
    support::run_native_files(&files).unwrap().unwrap_or(-1)
}

#[test]
fn irrev_functions_overwrite_and_scale_in_place() {
    let src = "irrev fn crush(x: &mut i64, y: &mut i64, z: &mut f64) {
        x = y + 1;
        y *= 3;
        y %= 5;
        z *= 2.5;
    }";
    let main = r#"
void crush(int64_t*, int64_t*, double*);
int main(void) {
    int64_t x = 100, y = 7;
    double z = 4.0;
    crush(&x, &y, &z);
    /* x = 8, y = (7 * 3) % 5 = 1, z = 10 */
    return (x == 8 && y == 1 && z == 10.0) ? 0 : 1;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_reversible_function_with_an_irrev_block_runs_forward() {
    let src = "fn mix(x: &mut i64, y: &mut i64) {
        x += 5;
        irrev { y = 0; }
        x += 1;
    }";
    let main = r#"
void mix(int64_t*, int64_t*);
int main(void) {
    int64_t x = 1, y = 9;
    mix(&x, &y);
    return (x == 7 && y == 0) ? 0 : 1;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn plain_functions_still_get_an_inverse_and_irreversible_ones_do_not() {
    let src = "fn add(x: &mut i64, k: &i64) { x += k; }
               irrev fn wipe(x: &mut i64) { x = 0; }
               fn leaky(x: &mut i64) { irrev { x = 0; } }";
    let ir = support::ir(src);
    assert!(ir.contains("define void @add_inv("));
    assert!(!ir.contains("@wipe_inv"));
    assert!(!ir.contains("@leaky_inv"));
}

#[test]
fn reversible_code_runs_forward_and_back_alongside_irrev_code() {
    let src = "fn add(x: &mut i64, k: &i64) { x += k; }
               irrev fn wipe(x: &mut i64) { x = 0; }
               fn both(a: &mut i64, b: &mut i64, k: &i64) {
                   call add(a, k);
                   irrev { call wipe(b); }
               }";
    let main = r#"
void both(int64_t*, int64_t*, int64_t*);
void add_inv(int64_t*, int64_t*);
int main(void) {
    int64_t a = 1, b = 9, k = 4;
    both(&a, &b, &k);
    if (a != 5 || b != 0) return 1;
    add_inv(&a, &k);
    return (a == 1) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}
