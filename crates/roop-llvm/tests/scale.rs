mod support;

use roop_llvm::Options;

fn run(main_c: &str) -> i32 {
    let src = "fn scale(x: &mut i64, k: &i64) { x *= k; }
               fn shrink(x: &mut i64, k: &i64) { x /= k; }
               fn grow(x: &mut f64, k: &f64) { x *= k; }";
    let out = support::compiled(src, &Options::default());
    let main = format!(
        "#include <stdint.h>\n\
         void scale(int64_t*, int64_t*); void scale_inv(int64_t*, int64_t*);\n\
         void shrink(int64_t*, int64_t*);\n\
         void grow(double*, double*); void grow_inv(double*, double*);\n{main_c}"
    );
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("main.c".to_string(), main.as_str()),
    ];
    support::run_native_files(&files).unwrap().unwrap_or(-1)
}

#[test]
fn scaling_multiplies_and_its_inverse_divides_exactly() {
    assert_eq!(
        run("int main(void) {
                 int64_t x = 6, k = 7;
                 scale(&x, &k);
                 if (x != 42) return 1;
                 scale_inv(&x, &k);
                 return x == 6 ? 0 : 2;
             }"),
        0
    );
}

#[test]
fn a_zero_factor_traps() {
    assert_eq!(
        run("int main(void) { int64_t x = 6, k = 0; scale(&x, &k); return 0; }"),
        -1
    );
    assert_eq!(
        run("int main(void) { int64_t x = 6, k = 0; shrink(&x, &k); return 0; }"),
        -1
    );
}

#[test]
fn an_overflowing_product_traps_instead_of_wrapping() {
    assert_eq!(
        run("int main(void) { int64_t x = 4611686018427387904, k = 2; scale(&x, &k); return 0; }"),
        -1
    );
    assert_eq!(
        run("int main(void) { int64_t x = INT64_MIN, k = -1; scale(&x, &k); return 0; }"),
        -1
    );
    assert_eq!(
        run("int main(void) { int64_t x = -3, k = -1; scale(&x, &k); return x == 3 ? 0 : 1; }"),
        0
    );
}

#[test]
fn an_inexact_division_traps_because_it_could_not_be_undone() {
    assert_eq!(
        run("int main(void) { int64_t x = 7, k = 2; shrink(&x, &k); return 0; }"),
        -1
    );
    assert_eq!(
        run("int main(void) { int64_t x = INT64_MIN, k = -1; shrink(&x, &k); return 0; }"),
        -1
    );
    assert_eq!(
        run("int main(void) { int64_t x = -8, k = 2; shrink(&x, &k); return x == -4 ? 0 : 1; }"),
        0
    );
}

#[test]
fn floats_scale_by_any_nonzero_factor() {
    assert_eq!(
        run("int main(void) {
                 double x = 4.0, k = 0.5;
                 grow(&x, &k);
                 if (x != 2.0) return 1;
                 grow_inv(&x, &k);
                 return x == 4.0 ? 0 : 2;
             }"),
        0
    );
    assert_eq!(
        run("int main(void) { double x = 4.0, k = 0.0; grow(&x, &k); return 0; }"),
        -1
    );
}
