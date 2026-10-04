mod support;

use roop_llvm::Options;

const SRC: &str = "fn fma(out: &mut i64, a: &i64, b: &i64) { out += a * b; }
                   fn add(x: &mut i64, y: &i64) { x += y; }
                   fn flip(x: &mut i64, y: &i64) { x += -y; }";

fn checked() -> Options {
    Options {
        check_overflow: true,
        ..Options::default()
    }
}

fn run(options: &Options, main_c: &str) -> i32 {
    let out = support::compiled(SRC, options);
    let main = format!(
        "#include <stdint.h>\n\
         extern volatile int64_t roop_overflow;\n\
         void fma(int64_t*, int64_t*, int64_t*); void fma_inv(int64_t*, int64_t*, int64_t*);\n\
         void add(int64_t*, int64_t*); void flip(int64_t*, int64_t*);\n{main_c}"
    );
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("main.c".to_string(), main.as_str()),
    ];
    support::run_native_files(&files).unwrap().unwrap_or(-1)
}

#[test]
fn a_wrapping_product_raises_the_flag_and_keeps_the_wrapped_result() {
    let big = "int main(void) {
        int64_t out = 1, a = 1LL << 40, b = 1LL << 40;
        fma(&out, &a, &b);
        return roop_overflow == 1 && out == 1 ? 0 : 1;
    }";
    assert_eq!(run(&checked(), big), 0);
}

#[test]
fn a_wrapping_sum_and_a_negated_minimum_raise_the_flag() {
    let sum = "int main(void) {
        int64_t x = INT64_MAX, y = 1;
        add(&x, &y);
        return roop_overflow == 1 && x == INT64_MIN ? 0 : 1;
    }";
    let neg = "int main(void) {
        int64_t x = 0, y = INT64_MIN;
        flip(&x, &y);
        return roop_overflow == 1 ? 0 : 1;
    }";
    assert_eq!(run(&checked(), sum), 0);
    assert_eq!(run(&checked(), neg), 0);
}

#[test]
fn arithmetic_that_fits_leaves_the_flag_down_and_the_result_exact() {
    let small = "int main(void) {
        int64_t out = 5, a = 1LL << 30, b = 1LL << 31;
        fma(&out, &a, &b);
        if (roop_overflow != 0 || out != 5 + (1LL << 61)) return 1;
        fma_inv(&out, &a, &b);
        return roop_overflow == 0 && out == 5 ? 0 : 2;
    }";
    assert_eq!(run(&checked(), small), 0);
}

#[test]
fn the_inverse_still_undoes_a_step_that_wrapped() {
    let wrapped = "int main(void) {
        int64_t out = 7, a = 1LL << 40, b = 3LL << 40;
        fma(&out, &a, &b);
        if (roop_overflow != 1) return 1;
        fma_inv(&out, &a, &b);
        return out == 7 ? 0 : 2;
    }";
    assert_eq!(run(&checked(), wrapped), 0);
}

#[test]
fn the_default_emits_no_flag_and_no_intrinsics() {
    let ir = support::ir(SRC);
    assert!(!ir.contains("roop_overflow") && !ir.contains("with.overflow"));
    let on = support::ir_with(SRC, &checked());
    assert!(on.contains("@roop_overflow") && on.contains("smul.with.overflow"));
}
