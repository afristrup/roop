mod support;

/// `main` returns 0 when every named condition holds.
fn harness(setup: &str, checks: &[&str]) -> String {
    let mut main = format!("define i32 @main() {{\n{setup}\n");
    let mut acc = "true".to_string();
    for (i, c) in checks.iter().enumerate() {
        main.push_str(&format!("  {c}\n  %all{i} = and i1 {acc}, %ok{i}\n"));
        acc = format!("%all{i}");
    }
    main.push_str(&format!(
        "  %bad = xor i1 {acc}, true\n  %r = zext i1 %bad to i32\n  ret i32 %r\n}}\n"
    ));
    main
}

fn expect_ok(src: &str, setup: &str, checks: &[&str]) {
    let ir = format!("{}\n{}", support::ir(src), harness(setup, checks));
    support::verify(&ir);
    if let Some(code) = support::run(&ir) {
        assert_eq!(code, Some(0), "{ir}");
    }
}

#[test]
fn update_swap_and_xor_reverse_exactly() {
    let src = "rev struct P { a: i64, b: i64 }
               rev fn step(p: &mut P, n: &i64) { p.a += n; p.b ^= n; p.a <=> p.b; }";
    let setup = "
  %p = alloca %P
  %pa = getelementptr inbounds %P, ptr %p, i32 0, i32 0
  %pb = getelementptr inbounds %P, ptr %p, i32 0, i32 1
  %n = alloca i64
  store i64 10, ptr %pa
  store i64 6, ptr %pb
  store i64 3, ptr %n
  call void @step(ptr %p, ptr %n)
  %a1 = load i64, ptr %pa
  %b1 = load i64, ptr %pb
  call void @step_inv(ptr %p, ptr %n)
  %a2 = load i64, ptr %pa
  %b2 = load i64, ptr %pb";
    expect_ok(
        src,
        setup,
        &[
            "%ok0 = icmp eq i64 %a1, 5",
            "%ok1 = icmp eq i64 %b1, 13",
            "%ok2 = icmp eq i64 %a2, 10",
            "%ok3 = icmp eq i64 %b2, 6",
        ],
    );
}

#[test]
fn from_loop_runs_and_reverses() {
    let src = "rev fn count(x: &mut i64, i: &mut i64, n: &i64) {
        from i == 0 { x += 2; } loop { i += 1; } until i == n;
    }";
    let setup = "
  %x = alloca i64
  %i = alloca i64
  %n = alloca i64
  store i64 100, ptr %x
  store i64 0, ptr %i
  store i64 4, ptr %n
  call void @count(ptr %x, ptr %i, ptr %n)
  %x1 = load i64, ptr %x
  %i1 = load i64, ptr %i
  call void @count_inv(ptr %x, ptr %i, ptr %n)
  %x2 = load i64, ptr %x
  %i2 = load i64, ptr %i";
    expect_ok(
        src,
        setup,
        &[
            "%ok0 = icmp eq i64 %x1, 110",
            "%ok1 = icmp eq i64 %i1, 4",
            "%ok2 = icmp eq i64 %x2, 100",
            "%ok3 = icmp eq i64 %i2, 0",
        ],
    );
}

#[test]
fn match_on_enum_selects_arm_both_ways() {
    let src = "rev enum S { A, B }
               rev fn f(s: &S, y: &mut i64) {
                   match s {
                       S::A => { y += 1; } assert y == 11;
                       S::B => { y += 2; } assert y == 12;
                   }
               }";
    let setup = "
  %s = alloca i32
  %y = alloca i64
  store i32 1, ptr %s
  store i64 10, ptr %y
  call void @f(ptr %s, ptr %y)
  %y1 = load i64, ptr %y
  call void @f_inv(ptr %s, ptr %y)
  %y2 = load i64, ptr %y";
    expect_ok(
        src,
        setup,
        &["%ok0 = icmp eq i64 %y1, 12", "%ok1 = icmp eq i64 %y2, 10"],
    );
}

#[test]
fn if_fi_and_ancilla_reverse() {
    let src = "rev fn f(x: &mut i64, y: &mut i64) {
        ancilla t: i64 = 0 { t += x; y += t; t -= x; }
        if y > 10 { x += 1; } else { x -= 1; } fi x > 0;
    }";
    let setup = "
  %x = alloca i64
  %y = alloca i64
  store i64 5, ptr %x
  store i64 7, ptr %y
  call void @f(ptr %x, ptr %y)
  %x1 = load i64, ptr %x
  %y1 = load i64, ptr %y
  call void @f_inv(ptr %x, ptr %y)
  %x2 = load i64, ptr %x
  %y2 = load i64, ptr %y";
    expect_ok(
        src,
        setup,
        &[
            "%ok0 = icmp eq i64 %x1, 6",
            "%ok1 = icmp eq i64 %y1, 12",
            "%ok2 = icmp eq i64 %x2, 5",
            "%ok3 = icmp eq i64 %y2, 7",
        ],
    );
}

#[test]
fn failed_exit_assertion_traps() {
    let src = "rev fn f(x: &mut i64) { if x > 0 { x += 1; } fi x < 0; }";
    let setup = "
  %x = alloca i64
  store i64 5, ptr %x
  call void @f(ptr %x)
  %ok0 = icmp eq i64 0, 0";
    let ir = format!(
        "{}\n{}",
        support::ir(src),
        harness(setup, &["%ok0 = icmp eq i64 0, 0"])
    );
    support::verify(&ir);
    if let Some(code) = support::run(&ir) {
        assert_ne!(code, Some(0));
    }
}

#[test]
fn borrowed_array_element_is_updated_in_place() {
    let src = "rev fn f(a: &mut [i64; 3], k: &i64) { borrow c = a[1] { c += k; } }";
    let setup = "
  %a = alloca [3 x i64]
  %a1p = getelementptr inbounds [3 x i64], ptr %a, i64 0, i64 1
  %k = alloca i64
  store i64 20, ptr %a1p
  store i64 5, ptr %k
  call void @f(ptr %a, ptr %k)
  %v = load i64, ptr %a1p";
    expect_ok(src, setup, &["%ok0 = icmp eq i64 %v, 25"]);
}
