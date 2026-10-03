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

const STACK4: &str = "typedef struct { int64_t len; int64_t data[4]; } Stack4;\n";

#[test]
fn push_moves_a_value_onto_the_stack_and_pop_moves_it_back() {
    let src = "fn park(x: &mut i64, s: &mut Stack<i64, 4>) { push s <- x; }
               fn fetch(x: &mut i64, s: &mut Stack<i64, 4>) { pop s -> x; }";
    let main = format!(
        r#"{STACK4}
void park(int64_t*, Stack4*);
void park_inv(int64_t*, Stack4*);
void fetch(int64_t*, Stack4*);
int main(void) {{
    Stack4 s = {{0}};
    int64_t x = 7, y = 8;
    park(&x, &s);
    if (x != 0 || s.len != 1 || s.data[0] != 7) return 1;
    park(&y, &s);
    if (s.len != 2 || s.data[1] != 8) return 2;
    int64_t z = 0;
    fetch(&z, &s);
    if (z != 8 || s.len != 1 || s.data[1] != 0) return 3;
    park_inv(&x, &s);
    return (x == 7 && s.len == 0 && s.data[0] == 0) ? 0 : 4;
}}
"#
    );
    assert_eq!(run(src, &main), 0);
}

#[test]
fn a_full_stack_and_an_empty_stack_are_failures() {
    let src = "fn park(x: &mut i64, s: &mut Stack<i64, 2>) { push s <- x; }
               fn fetch(x: &mut i64, s: &mut Stack<i64, 2>) { pop s -> x; }";
    let overflow = r#"
typedef struct { int64_t len; int64_t data[2]; } Stack2;
void park(int64_t*, Stack2*);
int main(void) {
    Stack2 s = {0};
    int64_t a = 1, b = 2, c = 3;
    park(&a, &s); park(&b, &s);
    park(&c, &s);
    return 0;
}
"#;
    assert_eq!(run(src, overflow), -1);
    let underflow = r#"
typedef struct { int64_t len; int64_t data[2]; } Stack2;
void fetch(int64_t*, Stack2*);
int main(void) {
    Stack2 s = {0};
    int64_t a = 0;
    fetch(&a, &s);
    return 0;
}
"#;
    assert_eq!(run(src, underflow), -1);
}

#[test]
fn logged_updates_destroy_values_reversibly() {
    let src = "fn crush(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 4>) {
                   logged h { x = y + 1; y %= 5; }
               }";
    let main = format!(
        r#"{STACK4}
void crush(int64_t*, int64_t*, Stack4*);
void crush_inv(int64_t*, int64_t*, Stack4*);
int main(void) {{
    Stack4 h = {{0}};
    int64_t x = 5, y = 7;
    crush(&x, &y, &h);
    if (x != 8 || y != 2 || h.len != 2 || h.data[0] != 5 || h.data[1] != 7) return 1;
    crush_inv(&x, &y, &h);
    if (x != 5 || y != 7 || h.len != 0 || h.data[0] != 0 || h.data[1] != 0) return 2;
    crush(&x, &y, &h);
    x = 99;
    crush_inv(&x, &y, &h);
    return 3;
}}
"#
    );
    assert_eq!(
        run(src, &main),
        -1,
        "undoing a state the function did not produce fails"
    );
}

#[test]
fn compute_copy_uncompute_cleans_the_history_up() {
    let src = "fn scale(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 4>) {
                   logged h { x = y * 2; }
               }
               fn keep(x: &mut i64, y: &mut i64, r: &mut i64) {
                   ancilla h: Stack<i64, 4> = empty {
                       call scale(x, y, h);
                       r += x;
                       uncall scale(x, y, h);
                   }
               }";
    let main = r#"
void keep(int64_t*, int64_t*, int64_t*);
void keep_inv(int64_t*, int64_t*, int64_t*);
int main(void) {
    int64_t x = 9, y = 5, r = 0;
    keep(&x, &y, &r);
    if (x != 9 || y != 5 || r != 10) return 1;
    keep_inv(&x, &y, &r);
    return (x == 9 && y == 5 && r == 0) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}
