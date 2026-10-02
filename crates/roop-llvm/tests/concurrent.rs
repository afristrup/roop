mod support;

use roop_llvm::Options;

/// Compiles `src`, links the C `main` and returns the exit code. The C side
/// arms an alarm so a hang shows up as a failure rather than a stuck test.
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
fn a_producer_and_a_consumer_exchange_messages_and_reverse_exactly() {
    let src = "rev fn pc(x: &mut i64, y: &mut i64, a: &mut i64, b: &mut i64) {
        chan c: i64 {
            #[concurrent] { send c <- x; send c <- y; }
            #[concurrent] { recv c -> a; recv c -> b; }
        }
    }";
    let main = r#"
void pc(int64_t*, int64_t*, int64_t*, int64_t*);
void pc_inv(int64_t*, int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 3, y = 4, a = 0, b = 0;
    pc(&x, &y, &a, &b);
    if (x != 0 || y != 0 || a != 3 || b != 4) return 1;
    pc_inv(&x, &y, &a, &b);
    return (x == 3 && y == 4 && a == 0 && b == 0) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_client_and_server_converse_over_two_channels() {
    // The client sends a request; the server adds ten and answers.
    let src = "rev fn serve(x: &mut i64, y: &mut i64, z: &mut i64) {
        chan req: i64 { chan rep: i64 {
            #[concurrent] { send req <- x; recv rep -> y; }
            #[concurrent] { recv req -> z; z += 10; send rep <- z; }
        } }
    }";
    let main = r#"
void serve(int64_t*, int64_t*, int64_t*);
void serve_inv(int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 32, y = 0, z = 0;
    serve(&x, &y, &z);
    if (x != 0 || y != 42 || z != 0) return 1;
    serve_inv(&x, &y, &z);
    return (x == 32 && y == 0 && z == 0) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn many_messages_keep_their_order() {
    let n = 16;
    let sends: String = (0..n)
        .map(|i| format!("send c <- a[{i}];"))
        .collect::<Vec<_>>()
        .join(" ");
    let recvs: String = (0..n)
        .map(|i| format!("recv c -> b[{i}];"))
        .collect::<Vec<_>>()
        .join(" ");
    let src = format!(
        "rev fn pipeline(a: &mut [i64; {n}], b: &mut [i64; {n}]) {{
            chan c: i64 {{ #[concurrent] {{ {sends} }} #[concurrent] {{ {recvs} }} }}
        }}"
    );
    let main = format!(
        r#"
void pipeline(int64_t*, int64_t*);
void pipeline_inv(int64_t*, int64_t*);
int main(void) {{
    alarm(10);
    int64_t a[{n}], b[{n}];
    for (int i = 0; i < {n}; i++) {{ a[i] = 100 + i; b[i] = 0; }}
    pipeline(a, b);
    for (int i = 0; i < {n}; i++) if (a[i] != 0 || b[i] != 100 + i) return 1;
    pipeline_inv(a, b);
    for (int i = 0; i < {n}; i++) if (a[i] != 100 + i || b[i] != 0) return 2;
    return 0;
}}
"#
    );
    assert_eq!(run(&src, &main), 0);
}

#[test]
fn independent_tasks_run_concurrently_and_join() {
    let src = "rev fn two(x: &mut i64, y: &mut i64, k: &i64) {
        #[concurrent] { x += k; x += k; }
        #[concurrent] { y += k; }
    }";
    let main = r#"
void two(int64_t*, int64_t*, int64_t*);
void two_inv(int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 1, y = 2, k = 5;
    two(&x, &y, &k);
    if (x != 11 || y != 7) return 1;
    two_inv(&x, &y, &k);
    return (x == 1 && y == 2) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}

const GUARDED: &str = "rev fn guarded(x: &mut i64, y: &mut i64, z: &mut i64, w: &mut i64) {
    try {
        chan c: i64 {
            #[concurrent] { x += 5; send c <- y; }
            #[concurrent] { recv c -> z; if z > 100 { w += 1; } fi w > 0; }
        }
    } catch_rollback { w += 7; }
}";

#[test]
fn a_failed_assertion_in_one_task_rolls_the_whole_group_back() {
    let main = r#"
void guarded(int64_t*, int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    /* y = 1: the consumer's assertion fails after the producer already ran. */
    int64_t x = 10, y = 1, z = 0, w = 0;
    guarded(&x, &y, &z, &w);
    /* Every variable the group touched is back; only the handler's effect stays. */
    return (x == 15 && y == 1 && z == 0 && w == 7) ? 0 : 1;
}
"#;
    assert_eq!(run(GUARDED, main), 0);
}

#[test]
fn a_group_whose_assertions_hold_commits_and_skips_the_handler() {
    let main = r#"
void guarded(int64_t*, int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 10, y = 500, z = 0, w = 0;
    guarded(&x, &y, &z, &w);
    return (x == 15 && y == 0 && z == 500 && w == 1) ? 0 : 1;
}
"#;
    assert_eq!(run(GUARDED, main), 0);
}

#[test]
fn a_task_that_fails_before_sending_does_not_leave_its_peer_waiting_forever() {
    let src = "rev fn stuck(x: &mut i64, y: &mut i64, z: &mut i64, w: &mut i64) {
        try {
            chan c: i64 {
                #[concurrent] { if x > 100 { w += 1; } fi w > 0; send c <- y; }
                #[concurrent] { recv c -> z; }
            }
        } catch_rollback { w += 3; }
    }";
    let main = r#"
void stuck(int64_t*, int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 0, y = 9, z = 0, w = 0;
    stuck(&x, &y, &z, &w);
    return (x == 0 && y == 9 && z == 0 && w == 3) ? 0 : 1;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_sequential_try_restores_what_it_wrote() {
    let src = "rev fn t(x: &mut i64, y: &mut i64, z: &mut i64) {
        try {
            y += 5;
            if x > 0 { y += 1; } fi y > 100;
        } catch_rollback { z += 1; }
    }";
    let main = r#"
void t(int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 1, y = 2, z = 0;
    t(&x, &y, &z);
    return (y == 2 && z == 1) ? 0 : 1;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_failure_inside_a_parallel_loop_rolls_back_arrays_written_by_worker_threads() {
    let src = "rev fn t(a: &mut [i64; 64], s: &mut i64, i: &mut i64) {
        try {
            #[parallel(cpu)] from i == 0 {
                a[i] += 10;
                if a[i] > 100 { a[i] += 1; } fi a[i] > 100;
            } loop { i += 1; } until i == 63;
        } catch_rollback { s += 1; }
    }";
    let main = r#"
void t(int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t a[64], s = 0, i = 0;
    for (int m = 0; m < 64; m++) a[m] = m;
    t(a, &s, &i);
    for (int m = 0; m < 64; m++) if (a[m] != m) return 1;
    return (s == 1 && i == 0) ? 0 : 2;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn nested_tries_roll_back_to_the_innermost_checkpoint() {
    let src = "rev fn t(x: &mut i64, y: &mut i64, z: &mut i64) {
        try {
            x += 1;
            try {
                y += 5;
                if x > 0 { y += 1; } fi y > 100;
            } catch_rollback { z += 1; }
            if x > 0 { z += 0; } fi x > 0;
        } catch_rollback { z += 100; }
    }";
    let main = r#"
void t(int64_t*, int64_t*, int64_t*);
int main(void) {
    alarm(10);
    int64_t x = 0, y = 0, z = 0;
    t(&x, &y, &z);
    /* The inner try rolled back (y restored, z += 1); the outer committed (x += 1). */
    return (x == 1 && y == 0 && z == 1) ? 0 : 1;
}
"#;
    assert_eq!(run(src, main), 0);
}

#[test]
fn a_function_containing_try_has_no_inverse() {
    let src = "rev fn t(x: &mut i64, y: &mut i64) {
        try { if x > 0 { y += 1; } fi y > 0; } catch_rollback { y += 2; }
    }
    rev fn u(x: &mut i64, y: &mut i64) { uncall t(x, y); }";
    let program = roop_syntax::parse(src).unwrap();
    roop_check::check(&program).unwrap();
    let err = roop_llvm::compile_all(&program, &Options::default())
        .err()
        .unwrap();
    assert!(err.to_string().contains("containing try backward"), "{err}");
}

#[test]
fn undeclared_inverse_symbol_is_not_emitted_for_try_functions() {
    let src = "rev fn t(x: &mut i64, y: &mut i64) {
        try { if x > 0 { y += 1; } fi y > 0; } catch_rollback { y += 2; }
    }";
    let ir = support::ir(src);
    assert!(ir.contains("define void @t("));
    assert!(!ir.contains("@t_inv"));
}
