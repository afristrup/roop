use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!(
        "fn f(a: &mut [i64; 8], b: &[i64; 8], m: &mut [[i64; 4]; 4], s: &mut i64, \
         k: &i64, n: &i64, i: &mut i64, j: &mut i64) {{ {body} }}"
    );
    check(&parse(&src).unwrap())
}

fn par(body: &str) -> Result<(), CheckError> {
    run(&format!(
        "#[parallel] from i == 0 {{ {body} }} loop {{ i += 1; }} until i == 7;"
    ))
}

#[test]
fn accepts_elementwise_update_with_shared_reads() {
    assert_eq!(par("a[i] += b[i]; a[i] += k;"), Ok(()));
    assert_eq!(par("a[i] += b[0];"), Ok(()));
}

#[test]
fn accepts_affine_private_indices() {
    assert_eq!(par("a[i + 1] += b[i];"), Ok(()));
    assert_eq!(par("a[2 * i] += b[i];"), Ok(()));
    assert_eq!(par("m[j][i] += 1;"), Ok(()));
}

#[test]
fn accepts_iteration_private_ancilla() {
    assert_eq!(
        par("ancilla t: i64 = 0 { t += b[i]; a[i] += t; t -= b[i]; }"),
        Ok(())
    );
}

#[test]
fn accepts_borrow_of_private_cell() {
    assert_eq!(par("borrow c = a[i] { c += b[i]; }"), Ok(()));
}

#[test]
fn rejects_reduction_into_shared_scalar() {
    assert!(matches!(
        par("s += b[i];"),
        Err(CheckError::ParallelWriteNotDisjoint { var, .. }) if var == "s"
    ));
}

#[test]
fn rejects_write_to_fixed_element() {
    assert!(matches!(
        par("a[0] += b[i];"),
        Err(CheckError::ParallelWriteNotDisjoint { .. })
    ));
}

#[test]
fn rejects_cross_iteration_read_of_written_array() {
    assert!(matches!(
        par("a[i] += a[i + 1];"),
        Err(CheckError::ParallelCrossIteration { .. })
    ));
    assert!(matches!(
        par("a[i] += a[0];"),
        Err(CheckError::ParallelCrossIteration { .. })
    ));
}

#[test]
fn rejects_writing_the_induction_variable_or_bounds() {
    assert!(matches!(
        par("i += 1;"),
        Err(CheckError::ParallelInductionWritten { .. })
    ));
    let src = "#[parallel] from i == 0 { n += 1; } loop { i += 1; } until i == n;";
    assert!(matches!(
        run(src),
        Err(CheckError::ParallelBoundModified { var, .. }) if var == "n"
    ));
}

#[test]
fn rejects_calls_passing_shared_places() {
    assert!(matches!(
        par("call g(s);"),
        Err(CheckError::ParallelWriteNotDisjoint { .. })
    ));
    assert_eq!(par("call g(a[i]);"), Ok(()));
}

#[test]
fn rejects_non_loop_and_unrecognized_loop_shape() {
    assert!(matches!(
        run("#[parallel] s += 1;"),
        Err(CheckError::ParallelNotLoop { .. })
    ));
    let down = "#[parallel] from i == 7 { a[i] += 1; } loop { i -= 1; } until i == 0;";
    assert!(matches!(
        run(down),
        Err(CheckError::ParallelLoopShape { .. })
    ));
    let zero_step = "#[parallel] from i == 0 { a[i] += 1; } loop { i += 0; } until i == 7;";
    assert!(matches!(
        run(zero_step),
        Err(CheckError::ParallelLoopShape { .. })
    ));
}

fn with_callee(call: &str) -> Result<(), CheckError> {
    let src = format!(
        "fn g(x: &mut i64, y: &i64) {{ x += y; }}
         fn f(a: &mut [i64; 8], s: &mut i64, k: &i64, i: &mut i64) {{
             #[parallel] from i == 0 {{ {call} }} loop {{ i += 1; }} until i == 7;
         }}"
    );
    check(&parse(&src).unwrap())
}

#[test]
fn a_call_writes_only_the_places_it_gives_to_mutable_parameters() {
    assert_eq!(with_callee("call g(a[i], k);"), Ok(()));
    assert_eq!(with_callee("call g(a[i], s);"), Ok(()));
    assert!(matches!(
        with_callee("call g(s, k);"),
        Err(CheckError::ParallelWriteNotDisjoint { var, .. }) if var == "s"
    ));
    assert!(matches!(
        with_callee("call g(a[i], a[0]);"),
        Err(CheckError::ParallelCrossIteration { .. }) | Err(CheckError::CallAliasing { .. })
    ));
}
