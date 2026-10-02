mod support;

#[test]
fn a_try_with_an_outcome_is_proved_to_undo_in_the_forward_direction() {
    let t = support::verified(
        "fn guarded(x: &mut i64, y: &mut i64, failed: &mut bool) {
            try {
                x += 5;
                y += x;
                if x > 100 { y -= 1; } else { y += 1; } fi x > 100;
            } catch_rollback {
                x -= 1;
            } -> failed;
         }",
    );
    assert_eq!(t.reversible, ["guarded"]);
    assert!(t.lean.contains("theorem Thm.\u{ab}guarded_inv_f\u{bb}"));
    assert!(
        !t.lean.contains("theorem Thm.\u{ab}guarded_f_inv\u{bb}"),
        "the inverse of a handler applies to states the body would not have failed on"
    );
}

#[test]
fn tries_compose_with_loops_calls_and_logged_blocks() {
    let t = support::verified(
        "fn drive(x: &mut i64, i: &mut i64, failed: &mut bool) {
            try {
                from i == 0 {
                    if i < 3 { x += 1; } else { x += 2; } fi i < 3;
                } loop { i += 1; } until i == 5;
            } catch_rollback {
                x ^= 255;
            } -> failed;
         }
         fn risky(x: &mut i64, y: &mut i64) {
            x += 1;
            y += x;
            if y > 50 { x += 100; } else { x -= 1000; } fi y > 50;
         }
         fn caller(x: &mut i64, y: &mut i64, failed: &mut bool) {
            try { x += 7; call risky(x, y); } catch_rollback { y ^= 1; } -> failed;
         }
         fn crush(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 1>, failed: &mut bool) {
            try { logged h { x = 5; y = 6; } } catch_rollback { x += 1000; } -> failed;
         }",
    );
    assert_eq!(t.reversible, ["drive", "risky", "caller", "crush"]);
}

#[test]
fn a_try_inside_a_loop_makes_the_loop_one_way() {
    let t = support::verified(
        "fn each(x: &mut i64, i: &mut i64, failed: &mut bool) {
            from i == 0 {
                try {
                    if i < 3 { x += 1; } else { x += 2; } fi i < 3;
                } catch_rollback {
                    x ^= 1;
                } -> failed;
                failed ^= true;
            } loop { i += 1; } until i == 5;
         }",
    );
    assert_eq!(t.reversible, ["each"]);
    assert!(!t.lean.contains("_loop_f_inv"));
}

#[test]
fn lean_rejects_a_handler_whose_inverse_is_wrong() {
    let mut t = support::translation(
        "fn guarded(x: &mut i64, failed: &mut bool) {
            try { if x > 0 { x += 1; } fi x > 0; } catch_rollback { x -= 1; } -> failed;
         }",
    );
    assert!(support::lean_accepts(&t).is_ok());
    let at = t
        .lean
        .find("_handler_inv\u{bb}")
        .expect("the inverse handler");
    let (head, tail) = t.lean.split_at(at);
    let tail = tail.replacen("(1 : Roop.I64)", "(2 : Roop.I64)", 1);
    t.lean = format!("{head}{tail}");
    assert!(
        support::lean_accepts(&t).is_err(),
        "the try is only undone if its handler is"
    );
}
