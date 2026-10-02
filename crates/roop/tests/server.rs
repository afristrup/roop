mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

const DRIVER: &str = r#"
#include <stdint.h>
typedef struct { int64_t place, offer, taken; } Conversation;
typedef struct { int64_t len; int64_t data[8]; } Stack8;
void converse(Conversation*, int64_t*, int64_t*, Stack8*);
void converse_inv(Conversation*, int64_t*, int64_t*, Stack8*);
void take(Conversation*, Stack8*);
void take_inv(Conversation*, Stack8*);

int main(void) {
    int64_t sea = 1, quiet = 0, better = 1;

    Conversation c = {0, 0, 0};
    Stack8 h = {0};
    converse(&c, &sea, &quiet, &h);
    if (c.place != 1 || c.offer != 1) return 1;

    converse_inv(&c, &sea, &quiet, &h);
    if (c.place != 0 || c.offer != 0 || h.len != 0) return 2;

    /* A better offer turns up: the house is rolled back and a bungalow offered. */
    converse(&c, &sea, &better, &h);
    if (c.place != 1 || c.offer != 2) return 3;
    take(&c, &h);
    if (c.taken != 2) return 4;

    /* The client abandons the conversation: everything is undone, in order. */
    take_inv(&c, &h);
    converse_inv(&c, &sea, &better, &h);
    if (c.place != 0 || c.offer != 0 || c.taken != 0 || h.len != 0) return 5;
    return 0;
}
"#;

fn std_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/std")
}

fn server_source() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/examples/server/mod.roop");
    std::fs::read_to_string(path).unwrap()
}

fn config() -> String {
    format!("[modules]\nstd = \"{}\"\n", std_dir().display())
}

#[test]
fn the_server_rolls_a_step_back_when_a_better_offer_turns_up() {
    let dir = project("server-run", &config(), &server_source());
    std::fs::write(dir.join("main.c"), DRIVER).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let code = Command::new(dir.join("prog")).status().unwrap().code();
    assert_eq!(code, Some(0), "step {code:?} of the C driver failed");
}

#[test]
fn lean_proves_every_step_and_the_whole_conversation_reversible() {
    let dir = project("server-lean", &config(), &server_source());
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for step in ["receive", "answer", "converse", "take"] {
        assert!(report.contains(step), "{step} missing: {report}");
    }
    assert!(report.contains("std__session__Negotiation"), "{report}");
    assert!(report.contains("Lean accepted the file"), "{report}");
}
