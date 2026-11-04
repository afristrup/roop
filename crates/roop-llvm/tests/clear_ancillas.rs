mod support;

use roop_llvm::Options;

const ADD: &str = "fn add(t: &mut [i64; 2], k: &i64) { t[0] += k; t[1] += k; }";

fn zeroing() -> Options {
    Options {
        clear_ancillas: true,
        ..Default::default()
    }
}

fn calls(ir: &str, callee: &str) -> usize {
    ir.matches(&format!("call void @{callee}(")).count()
}

#[test]
fn an_ancilla_a_call_made_is_zeroed_where_the_uncall_would_take_it_off() {
    let src = format!(
        "{ADD}
        fn user(x: &mut i64, k: &i64) {{
            ancilla t: [i64; 2] = 0;
            call add(t, k);
            x += t[0];
            uncall add(t, k);
        }}"
    );
    let ir = support::ir_with(&src, &zeroing());
    support::verify(&ir);
    assert_eq!(calls(&ir, "add"), 2, "{ir}");
    assert_eq!(calls(&ir, "add_inv"), 0, "{ir}");
    let computed = support::ir_with(&src, &Options::default());
    assert_eq!(calls(&computed, "add"), 2, "{computed}");
    assert_eq!(calls(&computed, "add_inv"), 2, "{computed}");
}

#[test]
fn a_callee_that_writes_more_than_the_ancilla_keeps_the_uncall() {
    let src = "fn two(t: &mut [i64; 2], u: &mut i64) { t[0] += 1; u += 1; }
        fn user(x: &mut i64, u: &mut i64) {
            ancilla t: [i64; 2] = 0;
            call two(t, u);
            x += t[0];
            uncall two(t, u);
        }";
    let ir = support::ir_with(src, &zeroing());
    assert_eq!(calls(&ir, "two_inv"), 2, "{ir}");
}

#[test]
fn only_the_outer_pair_of_calls_on_one_ancilla_is_zeroed() {
    let src = format!(
        "{ADD}
        fn user(x: &mut i64, k: &i64) {{
            ancilla t: [i64; 2] = 0;
            call add(t, k);
            call add(t, k);
            x += t[0];
            uncall add(t, k);
            uncall add(t, k);
        }}"
    );
    let ir = support::ir_with(&src, &zeroing());
    assert_eq!(calls(&ir, "add"), 4, "{ir}");
    assert_eq!(calls(&ir, "add_inv"), 2, "{ir}");
}

#[test]
fn a_zero_initialised_scalar_ancilla_uses_the_same_cleanup_witness() {
    let src = "fn add_scalar(t: &mut i64, k: &i64) { t += k; }
        fn user(x: &mut i64, k: &i64) {
            ancilla t: i64 = 0;
            call add_scalar(t, k);
            x += t;
            uncall add_scalar(t, k);
        }";
    let cleared = support::ir_with(src, &zeroing());
    support::verify(&cleared);
    assert_eq!(calls(&cleared, "add_scalar"), 2, "{cleared}");
    assert_eq!(calls(&cleared, "add_scalar_inv"), 0, "{cleared}");

    let conservative = support::ir(src);
    support::verify(&conservative);
    assert_eq!(calls(&conservative, "add_scalar_inv"), 2, "{conservative}");
}
