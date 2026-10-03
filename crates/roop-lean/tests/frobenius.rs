mod support;

fn file() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lean/Frobenius.lean")
}

#[test]
fn the_theory_roop_rests_on_checks_in_lean() {
    if let Err(report) = support::lean_checks(&file()) {
        panic!("{report}");
    }
}

#[test]
fn the_theory_is_proved_without_assumptions() {
    let text = std::fs::read_to_string(file()).unwrap();
    assert!(!text.contains("sorry"));
    for theorem in [
        "frobenius_law",
        "lists_are_not_frobenius",
        "kleisli_dagger_inverts_the_log",
        "reversing_a_try_keeps_the_outcome",
        "add_update_dagger",
    ] {
        assert!(text.contains(&format!("theorem {theorem}")), "{theorem}");
    }
}
