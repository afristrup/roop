use roop_check::check;
use roop_opt::fuse_parallel;
use roop_syntax::{Item, StmtKind, parse};

fn loops_after_fusion(src: &str) -> (usize, usize) {
    let program = parse(src).unwrap();
    check(&program).unwrap();
    let fused = fuse_parallel(&program);
    check(&fused).expect("fusion must preserve checkability");
    let Item::Fn(f) = fused.items.last().unwrap() else {
        panic!("expected fn")
    };
    let loops = f
        .body
        .stmts
        .iter()
        .filter(|s| matches!(s.kind, StmtKind::From { .. }))
        .count();
    (loops, f.body.stmts.len())
}

fn two_loops(second: &str) -> String {
    format!(
        "rev fn f(a: &mut [i64; 8], b: &mut [i64; 8], c: &[i64; 8], i: &mut i64, j: &mut i64) {{
            #[parallel] from i == 0 {{ a[i] += c[i]; }} loop {{ i += 1; }} until i == 7;
            {second}
        }}"
    )
}

#[test]
fn fuses_two_loops_over_the_same_range() {
    let src = two_loops("#[parallel] from j == 0 { b[j] += c[j]; } loop { j += 1; } until j == 7;");
    // assertion, fused loop, and the update that advances `j`
    assert_eq!(loops_after_fusion(&src), (1, 3));
}

#[test]
fn fuses_a_chain_of_three() {
    let src = "rev fn f(a: &mut [i64; 8], b: &mut [i64; 8], d: &mut [i64; 8], c: &[i64; 8],
                         i: &mut i64, j: &mut i64, k: &mut i64) {
        #[parallel] from i == 0 { a[i] += c[i]; } loop { i += 1; } until i == 7;
        #[parallel] from j == 0 { b[j] += c[j]; } loop { j += 1; } until j == 7;
        #[parallel] from k == 0 { d[k] += c[k]; } loop { k += 1; } until k == 7;
    }";
    assert_eq!(loops_after_fusion(src), (1, 5));
}

#[test]
fn fuses_when_the_second_loop_reads_the_first_loops_own_cell() {
    let src = two_loops("#[parallel] from j == 0 { b[j] += a[j]; } loop { j += 1; } until j == 7;");
    assert_eq!(loops_after_fusion(&src).0, 1);
}

#[test]
fn keeps_loops_apart_when_the_second_reads_a_neighbouring_cell() {
    let src =
        two_loops("#[parallel] from j == 0 { b[j] += a[j + 1]; } loop { j += 1; } until j == 6;");
    assert_eq!(loops_after_fusion(&src).0, 2);
}

#[test]
fn keeps_loops_apart_with_different_bounds_or_steps() {
    let bounds =
        two_loops("#[parallel] from j == 0 { b[j] += c[j]; } loop { j += 1; } until j == 6;");
    assert_eq!(loops_after_fusion(&bounds).0, 2);
    let step =
        two_loops("#[parallel] from j == 0 { b[j] += c[j]; } loop { j += 2; } until j == 6;");
    assert_eq!(loops_after_fusion(&step).0, 2);
}

#[test]
fn keeps_loops_apart_when_a_statement_sits_between_them() {
    let src = "rev fn f(a: &mut [i64; 8], b: &mut [i64; 8], c: &[i64; 8], s: &mut i64,
                        i: &mut i64, j: &mut i64) {
        #[parallel] from i == 0 { a[i] += c[i]; } loop { i += 1; } until i == 7;
        s += 1;
        #[parallel] from j == 0 { b[j] += c[j]; } loop { j += 1; } until j == 7;
    }";
    assert_eq!(loops_after_fusion(src).0, 2);
}

#[test]
fn keeps_loops_apart_when_they_disagree_on_the_target() {
    let src =
        "rev fn f(a: &mut [i64; 8], b: &mut [i64; 8], c: &[i64; 8], i: &mut i64, j: &mut i64) {
        #[parallel(cpu)] from i == 0 { a[i] += c[i]; } loop { i += 1; } until i == 7;
        #[parallel(metal)] from j == 0 { b[j] += c[j]; } loop { j += 1; } until j == 7;
    }";
    assert_eq!(loops_after_fusion(src).0, 2);
}

#[test]
fn leaves_sequential_loops_alone() {
    let src = "rev fn f(a: &mut [i64; 8], i: &mut i64, j: &mut i64) {
        from i == 0 { a[i] += 1; } loop { i += 1; } until i == 7;
        from j == 0 { a[j] += 1; } loop { j += 1; } until j == 7;
    }";
    assert_eq!(loops_after_fusion(src).0, 2);
}

#[test]
fn fuses_inside_nested_blocks() {
    let src =
        "rev fn f(a: &mut [i64; 8], b: &mut [i64; 8], c: &[i64; 8], i: &mut i64, j: &mut i64) {
        borrow z = c[0] {
            #[parallel] from i == 0 { a[i] += 1; } loop { i += 1; } until i == 7;
            #[parallel] from j == 0 { b[j] += 1; } loop { j += 1; } until j == 7;
        }
    }";
    let program = parse(src).unwrap();
    let fused = fuse_parallel(&program);
    let Item::Fn(f) = &fused.items[0] else {
        panic!()
    };
    let StmtKind::Borrow { body, .. } = &f.body.stmts[0].kind else {
        panic!()
    };
    assert_eq!(
        body.stmts
            .iter()
            .filter(|s| matches!(s.kind, StmtKind::From { .. }))
            .count(),
        1
    );
}
