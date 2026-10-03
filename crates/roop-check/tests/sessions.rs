use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(src: &str) -> Result<(), CheckError> {
    check(&parse(src).unwrap())
}

fn violation(src: &str) -> String {
    match run(src) {
        Err(CheckError::SessionNotCompliant { reason, path, .. }) => {
            format!("{reason} after {}", path.join(","))
        }
        other => panic!("expected a compliance failure, got {other:?}"),
    }
}

// The client of the paper: sea.(house + bung) + mount.house.
const CLIENT: &str = "offer { sea: offer { house: end, bung: end }, mount: offer { house: end } }";

#[test]
fn the_dual_of_a_behaviour_is_compliant_with_it() {
    let src = format!("session Booking {{ client: {CLIENT}; }}");
    assert_eq!(run(&src), Ok(()));
}

#[test]
fn a_server_that_may_send_what_the_client_does_not_offer_is_rejected() {
    // After `mount` the server may pick `bung`, but the client only offers `house`.
    let src = format!(
        "session Booking {{
            client: {CLIENT};
            server: select {{ mount: select {{ house: end, bung: end }} }};
        }}"
    );
    assert_eq!(
        violation(&src),
        "`bung` is sent but the other side does not offer it after mount"
    );
}

#[test]
fn a_server_may_use_less_than_the_client_offers() {
    let src = format!(
        "session Booking {{
            client: {CLIENT};
            server: select {{ sea: select {{ house: end }} }};
        }}"
    );
    assert_eq!(run(&src), Ok(()));
}

#[test]
fn two_parties_that_both_wait_or_both_choose_are_stuck() {
    let both_wait = "session S { client: offer { a: end }; server: offer { a: end }; }";
    assert!(violation(both_wait).contains("wait"));
    let both_choose = "session S { client: select { a: end }; server: select { a: end }; }";
    assert!(violation(both_choose).contains("choose"));
}

#[test]
fn a_finished_server_leaves_an_unfinished_client_stuck() {
    let src = "session S { client: select { a: end }; server: end; }";
    assert!(violation(src).contains("finished"));
}

#[test]
fn recursion_is_unfolded_and_checked_coinductively() {
    let src = "session Stream {
        client: rec again { select { more: again, stop: end } };
        server: rec again { offer { more: again, stop: end } };
    }";
    assert_eq!(run(src), Ok(()));
    let single = "session Stream { client: rec again { offer { more: again, stop: end } }; }";
    assert_eq!(run(single), Ok(()));
}

#[test]
fn matching_checkpoints_are_compliant() {
    let src = "session Booking {
        client: checkpoint offer { sea: checkpoint offer { house: end, bung: end },
                                   mount: offer { house: end } };
    }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn checkpoints_need_not_line_up_when_the_rollbacks_stay_compliant() {
    // The paper's example: the client asks for a house with garden, and the
    // server checkpoints in the middle of it.
    let src = "session Garden {
        client: checkpoint offer {
            sea: offer { house: offer { garden: end } },
            house: offer { garden: end },
        };
        server: checkpoint select {
            sea: checkpoint select { house: select { garden: end } },
            house: select { garden: end },
        };
    }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn a_side_that_can_roll_back_while_the_other_cannot_is_rejected() {
    let src = "session S {
        client: checkpoint offer { a: end };
        server: select { a: end };
    }";
    assert!(violation(src).contains("only one side has a checkpoint"));
}

#[test]
fn a_rollback_that_leads_into_a_mismatch_is_rejected() {
    // The server rolls back to a point where it may send `b`, which the
    // client never offers there.
    let src = "session S {
        client: offer { a: offer { c: end } };
        server: select { a: select { c: end } };
    }";
    assert_eq!(run(src), Ok(()));
    let src = "session S {
        client: checkpoint offer { a: offer { c: end } };
        server: checkpoint select { a: select { c: end }, b: end };
    }";
    assert!(violation(src).contains("`b`"));
}

#[test]
fn malformed_behaviours_are_reported() {
    let cases = [
        ("session S { client: x; }", "not bound"),
        ("session S { client: rec x { x }; }", "not guarded"),
        ("session S { client: offer { }; }", "at least one branch"),
        ("session S { client: offer { a: end, a: end }; }", "twice"),
        ("session S { client: end; client: end; }", "declared twice"),
        ("session S { a: end; b: end; c: end; }", "one or two roles"),
    ];
    for (src, expected) in cases {
        match run(src) {
            Err(CheckError::SessionMalformed { reason, .. }) => {
                assert!(reason.contains(expected), "{src}: {reason}")
            }
            other => panic!("{src}: {other:?}"),
        }
    }
}
