use crate::{
    AbortMode, CodegenError, Dialect, Dir, FnGen, Slot, chan_handle, copy_bytes, gen_block, layout,
};
use roop_check::{body_effects, channels_used};
use roop_syntax::{Block, Place, Type};

/// `try { body } catch_rollback { handler }`. The variables the body writes and
/// the channels it uses are checkpointed on entry. If an assertion fails
/// anywhere in the body, including inside its parallel loops and concurrent
/// tasks, all of them stop, the checkpoint is restored, and the handler runs.
/// A task group inside rolls back as one unit, which is the paper's
/// synchronous rollback: no task can undo its part while a peer keeps its own.
pub fn gen_try(
    g: &mut FnGen,
    body: &Block,
    handler: &Block,
    outcome: Option<&Place>,
    dir: Dir,
) -> Result<(), CodegenError> {
    if outcome.is_some() {
        return Err(CodegenError::Unsupported("try with an outcome"));
    }
    if dir == Dir::Backward {
        return Err(CodegenError::Unsupported(
            "reversing try ... catch_rollback",
        ));
    }
    if g.dialect != Dialect::Host {
        return Err(CodegenError::Unsupported("try inside a GPU kernel"));
    }
    let effects = body_effects(body);
    let mut saved: Vec<(Slot, String, u64)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (name, slot) in g.vars.clone().iter().rev() {
        let user = !name.starts_with("chan:") && !name.starts_with("__");
        if user && effects.writes.contains(name) && !seen.contains(name) {
            seen.push(name.clone());
            let size = layout(g.ctx, &slot.ty)?.0;
            let copy = g.alloca(&format!("[{size} x i8]"));
            copy_bytes(g, &copy, &slot.addr, size);
            saved.push((slot.clone(), copy, size));
        }
    }
    let mut snapshots: Vec<(String, String)> = Vec::new();
    for chan in channels_used(body) {
        if g.chan_types.iter().any(|(n, _)| *n == chan) {
            let (handle, _) = chan_handle(g, &chan)?;
            let snapshot = format!("%{}", g.fresh("t"));
            g.emit(&format!(
                "{snapshot} = call ptr @roop_chan_snapshot(ptr {handle})"
            ));
            let keep = g.alloca("ptr");
            g.emit(&format!("store ptr {snapshot}, ptr {keep}"));
            snapshots.push((chan, keep));
        }
    }

    let flag = g.alloca("i32");
    g.emit(&format!("store i32 0, ptr {flag}"));
    g.vars.push((
        "__abort".into(),
        Slot {
            addr: flag,
            ty: Type::Named("__abort".into()),
            space: 0,
        },
    ));
    let (rollback, finished, after) = (g.fresh("L"), g.fresh("L"), g.fresh("L"));
    let outer = std::mem::replace(&mut g.abort, AbortMode::Label(rollback.clone()));
    let result = gen_block(g, body, Dir::Forward);
    g.abort = outer;
    g.vars.pop();
    result?;
    g.emit(&format!("br label %{finished}"));

    g.label(&rollback);
    for (slot, copy, size) in &saved {
        copy_bytes(g, &slot.addr, copy, *size);
    }
    for (chan, keep) in &snapshots {
        let (handle, _) = chan_handle(g, chan)?;
        let snapshot = format!("%{}", g.fresh("t"));
        g.emit(&format!("{snapshot} = load ptr, ptr {keep}"));
        g.emit(&format!(
            "call void @roop_chan_restore(ptr {handle}, ptr {snapshot})"
        ));
    }
    free_snapshots(g, &snapshots);
    gen_block(g, handler, Dir::Forward)?;
    g.emit(&format!("br label %{after}"));

    g.label(&finished);
    free_snapshots(g, &snapshots);
    g.emit(&format!("br label %{after}"));
    g.label(&after);
    Ok(())
}

fn free_snapshots(g: &mut FnGen, snapshots: &[(String, String)]) {
    for (_, keep) in snapshots {
        let snapshot = format!("%{}", g.fresh("t"));
        g.emit(&format!("{snapshot} = load ptr, ptr {keep}"));
        g.emit(&format!(
            "call void @roop_chan_snapshot_free(ptr {snapshot})"
        ));
    }
}
