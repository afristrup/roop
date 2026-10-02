mod support;

use roop_llvm::Options;

fn run(src: &str, main_c: &str) -> i32 {
    let out = support::compiled(src, &Options::default());
    let main = format!("#include <stdint.h>\n#include <unistd.h>\n{main_c}");
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("main.c".to_string(), main.as_str()),
    ];
    support::run_native_files(&files).unwrap().unwrap_or(-1)
}

#[test]
fn a_failed_body_is_undone_by_running_it_backward_and_the_handler_sees_the_old_state() {
    let src = "fn guarded(x: &mut i64, y: &mut i64, failed: &mut bool) {
        try {
            x += 5;
            y += x;
            if x > 100 { y -= 1; } else { y += 1; } fi x > 100;
        } catch_rollback {
            x -= 1;
        } -> failed;
    }";
    let main = r#"
void guarded(int64_t*, int64_t*, uint8_t*);
void guarded_inv(int64_t*, int64_t*, uint8_t*);
int main(void) {
    int64_t x = 200, y = 1; uint8_t f = 0;
    guarded(&x, &y, &f);
    if (x != 205 || y != 205 || f != 0) return 1;
    guarded_inv(&x, &y, &f);
    if (x != 200 || y != 1 || f != 0) return 2;

    x = 3; y = 1;
    guarded(&x, &y, &f);
    if (x != 2 || y != 1 || f != 1) return 3;
    guarded_inv(&x, &y, &f);
    return (x == 3 && y == 1 && f == 0) ? 0 : 4;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_failure_in_a_later_iteration_undoes_the_finished_iterations() {
    let src = "fn drive(x: &mut i64, i: &mut i64, failed: &mut bool) {
        try {
            from i == 0 {
                if i < 3 { x += 1; } else { x += 2; } fi i < 3;
            } loop { i += 1; } until i == 5;
        } catch_rollback {
            x ^= 255;
        } -> failed;
    }";
    let main = r#"
void drive(int64_t*, int64_t*, uint8_t*);
void drive_inv(int64_t*, int64_t*, uint8_t*);
int main(void) {
    int64_t x = 10, i = 0; uint8_t f = 0;
    drive(&x, &i, &f);
    if (f != 1 || i != 0 || x != (10 ^ 255)) return 1;
    drive_inv(&x, &i, &f);
    return (x == 10 && i == 0 && f == 0) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_failing_callee_undoes_itself_before_the_try_handles_it() {
    let src = "fn risky(x: &mut i64, y: &mut i64) {
        x += 1;
        y += x;
        if y > 50 { x += 100; } else { x -= 1000; } fi y > 50;
    }
    fn caller(x: &mut i64, y: &mut i64, failed: &mut bool) {
        try {
            x += 7;
            call risky(x, y);
        } catch_rollback {
            y ^= 1;
        } -> failed;
    }";
    let main = r#"
void caller(int64_t*, int64_t*, uint8_t*);
void caller_inv(int64_t*, int64_t*, uint8_t*);
int main(void) {
    int64_t x = 1, y = 5; uint8_t f = 0;
    caller(&x, &y, &f);
    if (f != 1 || x != 1 || y != (5 ^ 1)) return 1;
    caller_inv(&x, &y, &f);
    if (x != 1 || y != 5 || f != 0) return 2;

    x = 1; y = 60;
    caller(&x, &y, &f);
    if (f != 0 || x != 109 || y != 69) return 3;
    caller_inv(&x, &y, &f);
    return (x == 1 && y == 60 && f == 0) ? 0 : 4;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_full_history_makes_a_logged_body_fail_and_roll_back() {
    let src = "fn crush(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 1>, failed: &mut bool) {
        try {
            logged h { x = 5; y = 6; }
        } catch_rollback {
            x += 1000;
        } -> failed;
    }";
    let main = r#"
typedef struct { int64_t len; int64_t data[1]; } Stack1;
void crush(int64_t*, int64_t*, Stack1*, uint8_t*);
void crush_inv(int64_t*, int64_t*, Stack1*, uint8_t*);
int main(void) {
    Stack1 h = {0};
    int64_t x = 1, y = 2; uint8_t f = 0;
    crush(&x, &y, &h, &f);
    if (f != 1 || x != 1001 || y != 2 || h.len != 0 || h.data[0] != 0) return 1;
    crush_inv(&x, &y, &h, &f);
    return (x == 1 && y == 2 && f == 0 && h.len == 0) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}
