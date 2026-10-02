use roop_check::check;
use roop_syntax::parse;
use std::fs;

fn check_file(relative: &str) {
    let path = format!("{}/../../roop/{relative}", env!("CARGO_MANIFEST_DIR"));
    let src = fs::read_to_string(&path).unwrap();
    let program = parse(&src).unwrap_or_else(|e| panic!("{relative}: {e}"));
    check(&program).unwrap_or_else(|e| panic!("{relative}: {e}"));
}

#[test]
fn std_module_is_valid() {
    check_file("std/mod.roop");
}

#[test]
fn insurance_example_is_valid() {
    check_file("examples/insurance/mod.roop");
}
