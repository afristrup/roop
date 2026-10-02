use roop_syntax::{Item, parse};

#[test]
fn parses_mod_use_struct_and_fn() {
    let src = "
        mod math;
        use std::num::Complex;
        rev struct Complex { re: f64, im: f64 }
        rev fn add(a: &mut Complex, b: &Complex) {
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
    let src = "rev fn f(xs: &mut [f64; 4]) { // note\n xs[0] += 1; }";
    assert!(parse(src).is_ok());
}
