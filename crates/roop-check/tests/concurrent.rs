use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!(
        "rev fn f(x: &mut i64, y: &mut i64, z: &mut i64, w: &mut i64, k: &i64) {{ {body} }}"
    );
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_a_producer_and_a_consumer() {
    let body = "chan c: i64 {
        #[concurrent] { send c <- x; send c <- y; }
        #[concurrent] { recv c -> z; recv c -> w; }
    }";
    assert_eq!(run(body), Ok(()));
}

#[test]
fn accepts_request_and_reply_over_two_channels() {
    // The client asks, the server answers: the paper's client/server shape.
    let body = "chan req: i64 { chan rep: i64 {
        #[concurrent] { send req <- x; recv rep -> y; }
        #[concurrent] { recv req -> z; send rep <- z; }
    } }";
    assert_eq!(run(body), Ok(()));
}

#[test]
fn accepts_tasks_that_only_read_shared_data() {
    let body = "#[concurrent] { x += k; } #[concurrent] { y += k; }";
    assert_eq!(run(body), Ok(()));
}

#[test]
fn rejects_two_tasks_writing_the_same_variable() {
    let body = "#[concurrent] { x += 1; } #[concurrent] { x += 2; }";
    assert!(matches!(
        run(body),
        Err(CheckError::ConcurrentConflict { var, .. }) if var == "x"
    ));
}

#[test]
fn rejects_a_task_reading_what_another_writes() {
    let body = "#[concurrent] { x += 1; } #[concurrent] { y += x; }";
    assert!(matches!(
        run(body),
        Err(CheckError::ConcurrentConflict { .. })
    ));
}

#[test]
fn rejects_a_receive_with_no_matching_send() {
    let body = "chan c: i64 { #[concurrent] { recv c -> y; } #[concurrent] { x += 1; } }";
    assert!(matches!(run(body), Err(CheckError::ChannelDeadlock { chan, .. }) if chan == "c"));
}

#[test]
fn rejects_a_send_nobody_receives() {
    let body = "chan c: i64 { #[concurrent] { send c <- x; } #[concurrent] { y += 1; } }";
    assert!(matches!(run(body), Err(CheckError::ChannelNotDrained { chan, .. }) if chan == "c"));
}

#[test]
fn rejects_more_sends_than_receives() {
    let body = "chan c: i64 {
        #[concurrent] { send c <- x; send c <- y; }
        #[concurrent] { recv c -> z; }
    }";
    assert!(matches!(
        run(body),
        Err(CheckError::ChannelNotDrained { .. })
    ));
}

#[test]
fn detects_a_deadlock_across_two_channels() {
    // Each side waits for the other to speak first.
    let body = "chan a: i64 { chan b: i64 {
        #[concurrent] { recv a -> x; send b <- y; }
        #[concurrent] { recv b -> z; send a <- w; }
    } }";
    assert!(matches!(run(body), Err(CheckError::ChannelDeadlock { .. })));
}

#[test]
fn rejects_two_senders_on_one_channel() {
    let body = "chan c: i64 {
        #[concurrent] { send c <- x; }
        #[concurrent] { send c <- y; }
        #[concurrent] { recv c -> z; recv c -> w; }
    }";
    assert!(matches!(
        run(body),
        Err(CheckError::ChannelNotPointToPoint { .. })
    ));
}

#[test]
fn rejects_channel_use_outside_a_task() {
    let body = "chan c: i64 { send c <- x; }";
    assert!(matches!(
        run(body),
        Err(CheckError::ChannelOpOutsideTask { .. })
    ));
}

#[test]
fn rejects_an_undeclared_channel() {
    let body = "#[concurrent] { send ghost <- x; }";
    assert!(matches!(run(body), Err(CheckError::UnknownChannel { .. })));
}

#[test]
fn rejects_channel_operations_whose_protocol_is_not_static() {
    let under_if = "chan c: i64 {
        #[concurrent] { if k > 0 { send c <- x; } fi k > 0; }
        #[concurrent] { recv c -> y; }
    }";
    assert!(matches!(
        run(under_if),
        Err(CheckError::ChannelOpInControlFlow { .. })
    ));
}

#[test]
fn a_task_may_not_roll_back_half_a_conversation() {
    // The paper's synchrony principle: rollback must involve both sides.
    let body = "chan c: i64 {
        #[concurrent] { try { send c <- x; if k > 0 { y += 1; } fi y > 0; } catch_rollback { } }
        #[concurrent] { recv c -> z; }
    }";
    assert!(matches!(
        run(body),
        Err(CheckError::ChannelOpInControlFlow { .. })
    ));
}

#[test]
fn rolling_back_the_whole_group_is_fine() {
    let body = "chan c: i64 { try {
        #[concurrent] { send c <- x; }
        #[concurrent] { recv c -> y; if k > 0 { z += 1; } fi z > 0; }
    } catch_rollback { } }";
    assert_eq!(run(body), Ok(()));
}

#[test]
fn rejects_concurrent_on_a_statement_that_is_not_a_block() {
    assert!(matches!(
        run("#[concurrent] x += 1;"),
        Err(CheckError::ConcurrentNotBlock { .. })
    ));
}

#[test]
fn channels_declared_inside_a_task_stay_private_to_it() {
    let body = "#[concurrent] { chan p: i64 {
            #[concurrent] { send p <- x; }
            #[concurrent] { recv p -> y; }
        } }
        #[concurrent] { z += 1; }";
    assert_eq!(run(body), Ok(()));
}

#[test]
fn ancilla_restoration_sees_sends_and_receives_as_inverses() {
    let body = "chan c: i64 {
        #[concurrent] { ancilla t: i64 = 0 { t += x; send c <- t; recv c -> t; t -= x; } }
    }";
    assert_eq!(run(body), Ok(()));
}
