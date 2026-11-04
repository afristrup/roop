mod support;

#[test]
fn a_leaf_function_stays_opaque_to_its_caller() {
    let calls = (0..32)
        .map(|_| "call add(x, k);")
        .collect::<Vec<_>>()
        .join(" ");
    let source = format!(
        "fn add(x: &mut i64, k: &i64) {{ x += k; }}\nfn many(x: &mut i64, k: &i64) {{ {calls} }}"
    );
    let translation = support::verified(&source);
    assert!(
        translation
            .lean
            .contains("attribute [irreducible] Fn.«add»")
    );
    assert!(translation.lean.contains("«add_inv_f_c»"));
}
