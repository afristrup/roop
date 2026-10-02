mod support;

use std::path::PathBuf;
use support::{project, roop, stderr};

fn std_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/std")
}

fn std_config() -> String {
    format!("[modules]\nstd = \"{}\"\n", std_dir().display())
}

#[test]
fn every_standard_session_is_checkpoint_compliant_and_proved_in_lean() {
    let text = std::fs::read_to_string(std_dir().join("session/mod.roop")).unwrap();
    let dir = project("std-sessions", "", &text);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in [
        "Booking",
        "Negotiation",
        "Garden",
        "Handshake",
        "Stream",
        "TwoPhase",
        "Request",
    ] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn importing_a_session_checks_it_and_a_broken_one_stops_the_build() {
    let good = project(
        "session-import",
        &std_config(),
        "use std::session::Booking;\nfn f(x: &mut i64) { x += 1; }",
    );
    assert!(
        roop(&good, &["build", "prog.roop", "--emit", "ir"])
            .status
            .success()
    );
    let report = String::from_utf8_lossy(&roop(&good, &["lean", "prog.roop"]).stdout).into_owned();
    assert!(report.contains("std__session__Booking"), "{report}");

    let bad = project(
        "session-bad",
        "",
        "session Booking {
            client: offer { sea: offer { house: end, bung: end }, mount: offer { house: end } };
            server: select { mount: select { house: end, bung: end } };
        }",
    );
    let out = roop(&bad, &["build", "prog.roop", "--emit", "ir"]);
    assert!(!out.status.success());
    let message = stderr(&out);
    assert!(message.contains("not checkpoint compliant"), "{message}");
    assert!(message.contains("`bung` is sent"), "{message}");
    assert!(message.contains("after mount"), "{message}");
}
