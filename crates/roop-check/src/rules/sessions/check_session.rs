use crate::{CheckError, compliant, dual, well_formed};
use roop_syntax::SessionDef;
use std::collections::HashSet;

/// A session is well formed, has one or two roles, and its client is
/// checkpoint compliant with its server (the dual of the client when only one
/// role is declared).
pub fn check_session(def: &SessionDef) -> Result<(), CheckError> {
    let malformed = |reason: String| CheckError::SessionMalformed {
        session: def.name.clone(),
        reason,
        span: def.span,
    };
    let mut names = HashSet::new();
    if let Some(role) = def.roles.iter().find(|r| !names.insert(&r.name)) {
        return Err(malformed(format!(
            "the role `{}` is declared twice",
            role.name
        )));
    }
    for role in &def.roles {
        well_formed(&role.behaviour)
            .map_err(|reason| malformed(format!("in role `{}`: {reason}", role.name)))?;
    }
    let (client, server) = match def.roles.as_slice() {
        [only] => (only.behaviour.clone(), dual(&only.behaviour)),
        [client, server] => (client.behaviour.clone(), server.behaviour.clone()),
        _ => return Err(malformed("a session has one or two roles".into())),
    };
    compliant(&client, &server).map_err(|violation| CheckError::SessionNotCompliant {
        session: def.name.clone(),
        path: violation.path,
        reason: violation.reason,
        span: def.span,
    })
}
