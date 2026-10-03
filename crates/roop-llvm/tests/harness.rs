#![allow(dead_code)]

pub const N: usize = 1000;

/// Counts mismatches of `a[m] == m * factor` over the whole array.
pub fn check_loop(name: &str, next: &str, factor: i64, prior: &str) -> String {
    format!(
        "  br label %{name}
{name}:
  %{name}m = phi i64 [0, %{prev}], [%{name}m1, %{name}]
  %{name}bad = phi i64 [{prior}, %{prev}], [%{name}bad1, %{name}]
  %{name}p = getelementptr inbounds [{N} x i64], ptr %a, i64 0, i64 %{name}m
  %{name}v = load i64, ptr %{name}p
  %{name}want = mul i64 %{name}m, {factor}
  %{name}ne = icmp ne i64 %{name}v, %{name}want
  %{name}nez = zext i1 %{name}ne to i64
  %{name}bad1 = add i64 %{name}bad, %{name}nez
  %{name}m1 = add i64 %{name}m, 1
  %{name}c = icmp slt i64 %{name}m1, {N}
  br i1 %{name}c, label %{name}, label %{next}
{next}:
",
        prev = if name == "chk1" { "run" } else { "back" }
    )
}

pub fn harness() -> String {
    let mut main = format!(
        "define i32 @main() {{
entry:
  %a = alloca [{N} x i64]
  %b = alloca [{N} x i64]
  %i = alloca i64
  %k = alloca i64
  store i64 0, ptr %i
  store i64 3, ptr %k
  br label %init
init:
  %n = phi i64 [0, %entry], [%n1, %init]
  %ap = getelementptr inbounds [{N} x i64], ptr %a, i64 0, i64 %n
  store i64 %n, ptr %ap
  %bp = getelementptr inbounds [{N} x i64], ptr %b, i64 0, i64 %n
  %n2 = mul i64 %n, 2
  store i64 %n2, ptr %bp
  %n1 = add i64 %n, 1
  %c = icmp slt i64 %n1, {N}
  br i1 %c, label %init, label %run
run:
  call void @axpy(ptr %a, ptr %b, ptr %i, ptr %k)
"
    );
    main.push_str(&check_loop("chk1", "back", 7, "0"));
    main.push_str("  call void @axpy_inv(ptr %a, ptr %b, ptr %i, ptr %k)\n");
    main.push_str(&check_loop("chk2", "done", 1, "%chk1bad1"));
    main.push_str(
        "  %iv = load i64, ptr %i
  %ivbad = icmp ne i64 %iv, 0
  %ivn = zext i1 %ivbad to i64
  %total = add i64 %chk2bad1, %ivn
  %r = trunc i64 %total to i32
  ret i32 %r
}
",
    );
    main
}
