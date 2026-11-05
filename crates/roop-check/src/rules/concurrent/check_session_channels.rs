use crate::{CheckError, channel_ops, child_blocks, dual, program_blocks, task_groups};
use roop_syntax::{Behaviour, Block, ChoiceKind, Program, SessionDef, Span, StmtKind};
use std::collections::BTreeMap;

/// Checks channels bound to a finite, linear session declaration.
pub fn check_session_channels(program: &Program) -> Result<(), CheckError> {
    let sessions: BTreeMap<&str, &SessionDef> = program
        .items
        .iter()
        .filter_map(|item| match item {
            roop_syntax::Item::Session(def) => Some((def.name.as_str(), def)),
            _ => None,
        })
        .collect();
    for block in program_blocks(program) {
        walk(block, &sessions)?;
    }
    Ok(())
}

fn walk(block: &Block, sessions: &BTreeMap<&str, &SessionDef>) -> Result<(), CheckError> {
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Chan {
                name,
                session: Some(session),
                body,
                ..
            } => {
                let Some(def) = sessions.get(session.as_str()) else {
                    return Err(binding(session, name, "is not declared", stmt.span));
                };
                let (client, server) =
                    roles(def).map_err(|reason| binding(session, name, &reason, stmt.span))?;
                tasks(name, body, &client, &server)
                    .map_err(|reason| binding(session, name, &reason, stmt.span))?;
                walk(body, sessions)?;
            }
            StmtKind::Chan { body, .. } => walk(body, sessions)?,
            _ => {
                for child in child_blocks(stmt) {
                    walk(child, sessions)?;
                }
            }
        }
    }
    Ok(())
}

fn binding(session: &str, chan: &str, reason: &str, span: Span) -> CheckError {
    CheckError::SessionTaskBinding {
        session: session.into(),
        chan: chan.into(),
        reason: reason.into(),
        span,
    }
}

fn roles(def: &SessionDef) -> Result<(Vec<bool>, Vec<bool>), String> {
    let (client, server) = match def.roles.as_slice() {
        [client] => (&client.behaviour, dual(&client.behaviour)),
        [client, server] => (&client.behaviour, server.behaviour.clone()),
        _ => return Err("must declare one or two roles".into()),
    };
    let client = trace(client)?;
    let server = trace(&server)?;
    if !one_way(&client) || !one_way(&server) {
        return Err(
            "must use one direction per role because a channel has one sender and one receiver"
                .into(),
        );
    }
    Ok((client, server))
}

fn one_way(trace: &[bool]) -> bool {
    trace.windows(2).all(|pair| pair[0] == pair[1])
}

fn trace(behaviour: &Behaviour) -> Result<Vec<bool>, String> {
    match behaviour {
        Behaviour::End => Ok(Vec::new()),
        Behaviour::Var(_) | Behaviour::Rec(_, _) => Err("must be finite and non-recursive".into()),
        Behaviour::Choice {
            kind,
            checkpoint,
            branches,
        } => {
            if *checkpoint {
                return Err("must not use checkpoints".into());
            }
            let [(label, next)] = branches.as_slice() else {
                return Err("must give every choice exactly one branch".into());
            };
            let mut out = vec![matches!(kind, ChoiceKind::Select)];
            out.extend(trace(next).map_err(|reason| format!("after `{label}`: {reason}"))?);
            Ok(out)
        }
    }
}

fn tasks(chan: &str, body: &Block, client: &[bool], server: &[bool]) -> Result<(), String> {
    let groups: Vec<_> = task_groups(body)
        .into_iter()
        .filter_map(|item| match item {
            crate::TaskItem::Group(tasks) => Some(tasks),
            crate::TaskItem::Single(_) => None,
        })
        .collect();
    let [tasks] = groups.as_slice() else {
        return Err("needs exactly one concurrent task group".into());
    };
    let [first, second] = tasks.as_slice() else {
        return Err("needs exactly two concurrent tasks".into());
    };
    let first = task_trace(first, chan)?;
    let second = task_trace(second, chan)?;
    if (first == client && second == server) || (first == server && second == client) {
        Ok(())
    } else {
        Err(format!(
            "task directions are {}, {} but the session needs {}, {}",
            show(&first),
            show(&second),
            show(client),
            show(server)
        ))
    }
}

fn task_trace(task: &roop_syntax::Stmt, chan: &str) -> Result<Vec<bool>, String> {
    channel_ops(
        match &task.kind {
            StmtKind::Block(block) => block,
            _ => return Err("has a non-block task".into()),
        },
        &mut Vec::new(),
    )
    .map(|ops| {
        ops.into_iter()
            .filter(|op| op.chan == chan)
            .map(|op| op.send)
            .collect()
    })
    .map_err(|_| "has channel operations in control flow".into())
}

fn show(trace: &[bool]) -> String {
    trace
        .iter()
        .map(|send| if *send { "send" } else { "recv" })
        .collect::<Vec<_>>()
        .join(" then ")
}
