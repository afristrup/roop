use crate::{CodegenError, FnGen};
use roop_syntax::Type;

/// Loads the runtime handle of a channel in scope and returns it with the
/// channel's message type.
pub fn chan_handle(g: &mut FnGen, name: &str) -> Result<(String, Type), CodegenError> {
    let unknown = || CodegenError::UnknownName {
        kind: "channel",
        name: name.into(),
    };
    let ty = g
        .chan_types
        .iter()
        .rev()
        .find(|(n, _)| n == name)
        .map(|(_, t)| t.clone())
        .ok_or_else(unknown)?;
    let key = format!("chan:{name}");
    let slot = g
        .vars
        .iter()
        .rev()
        .find(|(n, _)| *n == key)
        .map(|(_, s)| s.clone())
        .ok_or_else(unknown)?;
    let handle = format!("%{}", g.fresh("t"));
    g.emit(&format!("{handle} = load ptr, ptr {}", slot.addr));
    Ok((handle, ty))
}
