mod support;

const BOOKING: &str = "
session Booking {
    client: offer { sea: offer { house: end, bung: end }, mount: offer { house: end } };
}
session Garden {
    client: checkpoint offer {
        sea: offer { house: offer { garden: end } },
        house: offer { garden: end },
    };
    server: checkpoint select {
        sea: checkpoint select { house: select { garden: end } },
        house: select { garden: end },
    };
}
session Stream {
    client: rec again { select { more: again, stop: end } };
}
";

#[test]
fn lean_proves_checkpoint_compliance_of_declared_sessions() {
    let t = support::verified(BOOKING);
    assert_eq!(t.sessions, ["Booking", "Garden", "Stream"]);
    assert!(!t.lean.contains("sorry"));
}

#[test]
fn lean_rejects_a_pair_the_checker_would_have_refused() {
    let t = support::unchecked(
        "session Booking {
            client: offer { sea: offer { house: end, bung: end }, mount: offer { house: end } };
            server: select { mount: select { house: end, bung: end } };
        }",
    );
    assert!(
        support::lean_accepts(&t).is_err(),
        "Lean must not prove a non-compliant pair"
    );
}

#[test]
fn lean_rejects_a_rollback_that_only_one_side_can_do() {
    let t = support::unchecked(
        "session S { client: checkpoint offer { a: end }; server: select { a: end }; }",
    );
    assert!(support::lean_accepts(&t).is_err());
}
