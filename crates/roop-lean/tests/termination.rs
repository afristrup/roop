mod support;

#[test]
fn a_loop_over_numbers_and_arrays_gets_a_termination_theorem() {
    let t = support::verified(
        "fn count(a: &mut [i64; 4], b: &mut bool) {
            ancilla i: i64 = 0 {
                from i == 0 { a[0] += 1; } loop { i += 1; } until i == 3;
                i -= 3;
            }
         }",
    );
    assert!(t.lean.contains("loop_terminates"), "{}", t.lean);
    assert!(t.lean.contains("Roop.janus_terminates"));
}

#[test]
fn a_loop_over_a_stack_does_not() {
    let t = support::verified(
        "fn keep(x: &mut i64, h: &mut Stack<i64, 4>) {
            ancilla i: i64 = 0 {
                from i == 0 { push h <- x; pop h -> x; } loop { i += 1; } until i == 3;
                i -= 3;
            }
         }",
    );
    assert!(!t.lean.contains("loop_terminates"));
}

#[test]
fn the_termination_theorem_rests_on_the_usual_axioms_only() {
    let t = support::translation("fn id(x: &mut i64) { x += 0; }");
    let mut file = t.lean.clone();
    file.push_str(
        "
#print axioms Roop.janus_terminates
#print axioms Roop.distinct_bounded
example : Roop.Coded ((Vector Roop.I64 3) \u{d7} Bool \u{d7} Roop.I64) := inferInstance
",
    );
    let Some(out) = support::lean_output(&file) else {
        return;
    };
    assert!(!out.contains("sorryAx"), "{out}");
    assert!(!out.contains("error"), "{out}");
    assert!(out.contains("Classical.choice"), "{out}");
}

#[test]
fn a_captured_variable_named_a_does_not_clash_with_the_theorem() {
    let t = support::verified(
        "fn spread(x: &mut i64, a: &i64) {
            ancilla i: i64 = 0 {
                from i == 0 { x += a; } loop { i += 1; } until i == 3;
                i -= 3;
            }
         }",
    );
    assert!(t.lean.contains("loop_terminates"));
}

#[test]
fn captured_variables_named_n_and_s_do_not_clash_either() {
    let t = support::verified(
        "fn count(x: &mut i64, i: &mut i64, n: &i64, s: &i64) {
            from i == 0 { x += s; } loop { i += 1; } until i == n;
         }",
    );
    assert!(t.lean.contains("loop_terminates"));
}
