use crate::{
    CodegenError, Dialect, Dir, FnGen, ParallelLoop, Value, bool_type, device_buffers, gen_assert,
    gen_kernel, layout,
};
use roop_check::body_effects;
use roop_syntax::Block;

/// Describes each captured variable as `{ptr, size, written}`, then asks the
/// runtime to copy them to the device, run one thread per iteration, and
/// copy the written ones back.
pub fn launch_gpu(
    g: &mut FnGen,
    dialect: Dialect,
    pl: &ParallelLoop,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    let effects = body_effects(body);
    let buffers = device_buffers(g, &pl.var, &effects);
    let kernel = gen_kernel(g, dialect, &pl.var, body, dir, &buffers)?;

    let n = buffers.len();
    let table = g.alloca(&format!("[{n} x %RoopBuf]"));
    for (i, (_, slot, writable)) in buffers.iter().enumerate() {
        let (size, _) = layout(g.ctx, &slot.ty)?;
        let entry = format!("%{}", g.fresh("t"));
        g.emit(&format!(
            "{entry} = getelementptr inbounds [{n} x %RoopBuf], ptr {table}, i64 0, i64 {i}"
        ));
        for (field, ty, value) in [
            (0, "ptr", slot.addr.clone()),
            (1, "i64", size.to_string()),
            (2, "i32", u8::from(*writable).to_string()),
        ] {
            let at = format!("%{}", g.fresh("t"));
            g.emit(&format!(
                "{at} = getelementptr inbounds %RoopBuf, ptr {entry}, i32 0, i32 {field}"
            ));
            g.emit(&format!("store {ty} {value}, ptr {at}"));
        }
    }
    let label = format!("@{kernel}.name");
    let bytes = kernel.len() + 1;
    g.globals.push(format!(
        "{label} = private constant [{bytes} x i8] c\"{kernel}\\00\"\n"
    ));
    let (kind, blob, blob_len) = match dialect {
        Dialect::Air => {
            let len = format!("%{}", g.fresh("t"));
            g.emit(&format!("{len} = load i64, ptr @roop_metallib_len"));
            (0, "@roop_metallib", len)
        }
        _ => (1, "@roop_ptx", "0".to_string()),
    };
    let status = format!("%{}", g.fresh("t"));
    g.emit(&format!(
        "{status} = call i32 @roop_gpu_dispatch(i32 {kind}, ptr {blob}, i64 {blob_len}, ptr {label}, ptr {table}, i64 {n}, i64 {}, i64 {}, i64 {})",
        pl.space.lo.reg, pl.step, pl.space.count
    ));
    let ok = format!("%{}", g.fresh("t"));
    g.emit(&format!("{ok} = icmp eq i32 {status}, 0"));
    gen_assert(
        g,
        &Value {
            reg: ok,
            ty: bool_type(),
        },
    )
}
