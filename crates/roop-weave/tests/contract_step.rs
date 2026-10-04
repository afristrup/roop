mod support;

use support::{project, roop, text};

/// Runs a main whose exit status is 0 when `check` holds after `call` has run on the
/// arrays that `setup` makes.
fn holds(name: &str, setup: &str, call: &str, check: &str) -> bool {
    let dir = project(name);
    let program = format!(
        "use weave::*;

irrev fn run(status: &mut i64) {{
{setup}
    {call}
    if {check} {{ status = 0; }} else {{ status = 1; }} fi {check};
}}

fn main(status: &mut i64) {{
    irrev {{ call run(status); }}
}}
"
    );
    std::fs::write(dir.join("prog.roop"), program).unwrap();
    let out = roop(&dir, &["run", "prog.roop"]);
    assert!(out.status.code().is_some(), "{}", text(&out));
    out.status.success()
}

const WEIGHTS: &str = "    ancilla w1: [[i64; 2]; 2] = 0;
    ancilla w2: [[i64; 2]; 2] = 0;
    w1[0][0] += 8192;
    w1[1][1] += 4096;
    w2[0][0] += 2048;
    w2[1][1] += 2048;";

#[test]
fn weights_whose_bound_is_below_the_cap_are_left_alone() {
    let call = "call project_contraction<2, 2>(w1, w2, 4096, 6144);";
    let check = "w1[0][0] == 8192 && w1[1][1] == 4096 && w2[0][0] == 2048 && w2[1][1] == 2048";
    assert!(holds("keep_below", WEIGHTS, call, check));
}

#[test]
fn weights_whose_bound_is_above_the_cap_shrink_to_it_in_proportion() {
    let call = "call project_contraction<2, 2>(w1, w2, 4096, 2048);";
    let check = "w1[0][0] * w2[0][0] / 4096 <= 2048 && w1[0][0] * w2[0][0] / 4096 >= 1900 && w1[1][1] * 2 <= w1[0][0] + 2 && w1[1][1] * 2 + 2 >= w1[0][0]";
    assert!(holds("keep_above", WEIGHTS, call, check));
}

#[test]
fn a_step_below_a_unit_is_carried_and_not_lost_for_either_sign() {
    let setup = "    ancilla w: [i64; 2] = 0;
    ancilla g: [i64; 2] = 0;
    ancilla c: [i64; 2] = 0;
    g[0] -= 5000;
    g[1] += 5000;";
    let call =
        "call sgd_carry_vec<2>(w, g, c, 4096, 4096); call sgd_carry_vec<2>(w, g, c, 4096, 4096);";
    let check = "w[0] == 2 && c[0] == 0 - 1808 && w[1] == 0 - 2 && c[1] == 1808";
    assert!(holds("carry", setup, call, check));
}

const OUTWARD: &str = "    ancilla w1: [[i64; 2]; 2] = 0;
    ancilla w2: [[i64; 2]; 2] = 0;
    ancilla g1: [[i64; 2]; 2] = 0;
    ancilla g2: [[i64; 2]; 2] = 0;
    w1[0][0] += 8192;
    w1[1][1] += 4096;
    w2[0][0] += 2048;
    w2[1][1] += 2048;
    g1[0][0] -= 4194304;";

#[test]
fn an_outward_gradient_at_the_bound_loses_its_part_along_the_normal() {
    let call = "call project_gradient<2, 2>(g1, g2, w1, w2, 4096, 4096);";
    let check = "g1[0][0] < 0 - 3600000 && g1[0][0] > 0 - 3800000 && g2[0][0] > 800000 && g2[0][0] < 1000000 && g2[1][1] > 800000 && g1[1][1] == 0";
    assert!(holds("grad_outward", OUTWARD, call, check));
}

#[test]
fn an_inward_gradient_at_the_bound_is_left_alone() {
    let setup = format!("{OUTWARD}\n    g1[0][0] += 8388608;");
    let call = "call project_gradient<2, 2>(g1, g2, w1, w2, 4096, 4096);";
    assert!(holds(
        "grad_inward",
        &setup,
        call,
        "g1[0][0] == 4194304 && g2[0][0] == 0"
    ));
}

#[test]
fn a_gradient_below_the_bound_is_left_alone() {
    let call = "call project_gradient<2, 2>(g1, g2, w1, w2, 4096, 8192);";
    assert!(holds(
        "grad_below",
        OUTWARD,
        call,
        "g1[0][0] == 0 - 4194304 && g2[0][0] == 0"
    ));
}
