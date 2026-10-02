use roop_syntax::{Item, parse};

#[test]
fn parses_mod_use_struct_and_fn() {
    let src = "
        mod math;
        use std::num::Complex;
        struct Complex { re: f64, im: f64 }
        fn add(a: &mut Complex, b: &Complex) {
            a.re += b.re;
            a.im += b.im;
        }
    ";
    let program = parse(src).unwrap();
    assert!(matches!(program.items[0], Item::Mod(_)));
    assert!(matches!(&program.items[1], Item::Use(p) if p.len() == 3));
    assert!(matches!(program.items[2], Item::Struct(_)));
    assert!(matches!(program.items[3], Item::Fn(_)));
}

#[test]
fn parses_array_types_and_comments() {
    let src = "fn f(xs: &mut [f64; 4]) { // note\n xs[0] += 1; }";
    assert!(parse(src).is_ok());
}

#[test]
fn parses_struct_with_build_and_unbuild() {
    let src = "
        struct Complex {
            re: f64,
            im: f64,
            build(r: &f64, i: &f64) { self.re += r; self.im += i; }
            unbuild(r: &f64, i: &f64) { self.im -= i; self.re -= r; }
        }
    ";
    let program = parse(src).unwrap();
    let Item::Struct(def) = &program.items[0] else {
        panic!("expected struct")
    };
    assert_eq!(def.fields.len(), 2);
    assert!(def.build.is_some() && def.unbuild.is_some());
}

#[test]
fn parses_struct_without_constructors() {
    assert!(parse("struct P { a: i64 }").is_ok());
}

#[test]
fn rejects_unbuild_before_build() {
    let src = "struct P { a: i64, unbuild() { } build() { } }";
    assert!(parse(src).is_err());
}

#[test]
fn parses_enum_and_variant_uses() {
    let src = "
        enum Signal { High, Low, }
        fn f(s: &Signal, y: &mut i64) {
            match s {
                Signal::High => { y += 1; } assert s == Signal::High;
                _ => { } assert s != Signal::High;
            }
        }
    ";
    let program = parse(src).unwrap();
    let Item::Enum(def) = &program.items[0] else {
        panic!("expected enum")
    };
    assert_eq!(def.variants, ["High", "Low"]);
}
