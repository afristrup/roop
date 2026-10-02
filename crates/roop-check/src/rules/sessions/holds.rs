use crate::{Config, Violation, mark, unfold};
use roop_syntax::{Behaviour, ChoiceKind};
use std::collections::HashSet;

fn violation(path: &[String], reason: impl Into<String>) -> Violation {
    Violation {
        path: path.to_vec(),
        reason: reason.into(),
    }
}

/// One judgment of the paper's system: the configuration is checkpoint
/// compliant if it is already assumed on the way here, or the client is done
/// and the pasts agree, or the two choices match up and every action leads to a
/// compliant configuration, and so does a rollback if there is one to roll back.
pub fn holds(
    seen: &mut HashSet<Config>,
    path: &mut Vec<String>,
    client_past: Option<&Behaviour>,
    client: &Behaviour,
    server_past: Option<&Behaviour>,
    server: &Behaviour,
) -> Result<(), Violation> {
    let (client, server) = (unfold(client), unfold(server));
    let config = Config {
        client_past: client_past.cloned(),
        client: client.clone(),
        server_past: server_past.cloned(),
        server: server.clone(),
    };
    if !seen.insert(config.clone()) {
        return Ok(());
    }
    let rolls_back =
        |seen: &mut HashSet<Config>, path: &mut Vec<String>| match (client_past, server_past) {
            (None, None) => Ok(()),
            (Some(c), Some(s)) => {
                path.push("rollback".into());
                let result = holds(seen, path, None, c, None, s);
                path.pop();
                result
            }
            _ => Err(violation(
                path,
                "only one side has a checkpoint to roll back to",
            )),
        };
    match (&client, &server) {
        (Behaviour::End, _) => rolls_back(seen, path),
        (
            Behaviour::Choice {
                kind: client_kind,
                branches: client_branches,
                ..
            },
            Behaviour::Choice {
                kind: server_kind,
                branches: server_branches,
                ..
            },
        ) if client_kind != server_kind => {
            // The party that selects must be understood by the one that offers.
            let (selects, offers) = match client_kind {
                ChoiceKind::Select => (client_branches, server_branches),
                ChoiceKind::Offer => (server_branches, client_branches),
            };
            if let Some((label, _)) = selects
                .iter()
                .find(|(label, _)| !offers.iter().any(|(other, _)| other == label))
            {
                return Err(violation(
                    path,
                    format!("`{label}` is sent but the other side does not offer it"),
                ));
            }
            let (client_next, server_next) = (
                mark(&client_past.cloned(), &client),
                mark(&server_past.cloned(), &server),
            );
            for (label, selected) in selects {
                let offered = &offers
                    .iter()
                    .find(|(other, _)| other == label)
                    .expect("checked")
                    .1;
                let (c, s) = match client_kind {
                    ChoiceKind::Select => (selected, offered),
                    ChoiceKind::Offer => (offered, selected),
                };
                path.push(label.clone());
                let result = holds(seen, path, client_next.as_ref(), c, server_next.as_ref(), s);
                path.pop();
                result?;
            }
            rolls_back(seen, path)
        }
        (Behaviour::Choice { .. }, Behaviour::Choice { kind, .. }) => Err(violation(
            path,
            format!(
                "both sides {} at the same time",
                match kind {
                    ChoiceKind::Offer => "wait",
                    ChoiceKind::Select => "choose",
                }
            ),
        )),
        (Behaviour::Choice { .. }, Behaviour::End) => Err(violation(
            path,
            "the server is finished but the client still has something to do",
        )),
        _ => Err(violation(path, "a recursion variable was left unbound")),
    }
}
