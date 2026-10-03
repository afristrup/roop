use roop_syntax::{Behaviour, ChoiceKind, Item, parse};

fn only_session(src: &str) -> roop_syntax::SessionDef {
    let program = parse(src).unwrap();
    match program.items.into_iter().next() {
        Some(Item::Session(def)) => def,
        other => panic!("expected a session, got {other:?}"),
    }
}

#[test]
fn parses_choices_checkpoints_and_recursion() {
    let def = only_session(
        "pub session S {
            client: checkpoint offer { a: end, b: again };
            server: rec again { select { a: end } };
        }",
    );
    assert!(def.public);
    assert_eq!(def.roles.len(), 2);
    assert!(matches!(
        &def.roles[0].behaviour,
        Behaviour::Choice { kind: ChoiceKind::Offer, checkpoint: true, branches } if branches.len() == 2
    ));
    assert!(matches!(&def.roles[1].behaviour, Behaviour::Rec(name, _) if name == "again"));
}

#[test]
fn the_behaviour_words_stay_usable_as_names() {
    let program = parse("fn f(offer: &mut i64, select: &i64, end: &i64) { offer += select; }");
    assert!(program.is_ok());
}

#[test]
fn rejects_sessions_without_roles() {
    assert!(parse("session S { }").is_err());
}
