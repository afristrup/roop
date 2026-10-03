mod support;

use std::fs;

fn example(relative: &str) -> String {
    fs::read_to_string(format!(
        "{}/../../roop/{relative}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn std_module_is_valid_ir() {
    support::verify(&support::ir(&example("std/mod.roop")));
}

#[test]
fn insurance_example_is_valid_ir() {
    let ir = support::ir(&example("examples/insurance/mod.roop"));
    assert!(ir.contains("define void @settle("));
    assert!(ir.contains("define void @settle_inv("));
    assert!(ir.contains("define void @Policy_build("));
    support::verify(&ir);
}
