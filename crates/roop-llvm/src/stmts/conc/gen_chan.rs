use crate::{
    CodegenError, Dialect, Dir, FnGen, Slot, Value, bool_type, gen_assert, gen_block, layout,
};
use roop_syntax::{Block, Type};

/// A channel lives for its body. Its handle sits in a `chan:NAME` variable so
/// the tasks of the body can capture it, and it must be empty at the end.
pub fn gen_chan(
    g: &mut FnGen,
    name: &str,
    ty: &Type,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    if g.dialect != Dialect::Host {
        return Err(CodegenError::Unsupported("channels inside a GPU kernel"));
    }
    let size = layout(g.ctx, ty)?.0;
    let handle = format!("%{}", g.fresh("t"));
    g.emit(&format!("{handle} = call ptr @roop_chan_new(i64 {size})"));
    let slot = g.alloca("ptr");
    g.emit(&format!("store ptr {handle}, ptr {slot}"));
    g.vars.push((
        format!("chan:{name}"),
        Slot {
            addr: slot,
            ty: Type::Named("__chan".into()),
            space: 0,
        },
    ));
    g.chan_types.push((name.into(), ty.clone()));
    let result = gen_block(g, body, dir);
    g.chan_types.pop();
    g.vars.pop();
    result?;
    let left = format!("%{}", g.fresh("t"));
    g.emit(&format!("{left} = call i64 @roop_chan_len(ptr {handle})"));
    let empty = format!("%{}", g.fresh("t"));
    g.emit(&format!("{empty} = icmp eq i64 {left}, 0"));
    gen_assert(
        g,
        &Value {
            reg: empty,
            ty: bool_type(),
        },
    )?;
    g.emit(&format!("call void @roop_chan_free(ptr {handle})"));
    Ok(())
}
