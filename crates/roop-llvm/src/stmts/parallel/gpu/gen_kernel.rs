use crate::{
    CodegenError, Dialect, Dir, FnGen, Kernel, KernelArg, Slot, gen_block, kernel_prologue, layout,
    llvm_type, mem_store, params_space, ptr_type,
};
use roop_syntax::{Block, Type};

/// Turns a loop body into a device kernel. Every captured variable becomes a
/// device buffer; the host runtime copies them in and the written ones back.
/// Returns the kernel's name.
pub fn gen_kernel(
    parent: &mut FnGen,
    dialect: Dialect,
    var: &str,
    body: &Block,
    dir: Dir,
    buffers: &[(String, Slot, bool)],
) -> Result<String, CodegenError> {
    let id = parent.fresh("");
    let name = format!("{}_k{id}", parent.symbol.replace('.', "_"));
    let mut k = FnGen::new(parent.ctx, name.clone(), dialect);
    k.error_flag = Some("%err".into());

    let mut params = Vec::new();
    let mut args = Vec::new();
    for (i, (var_name, slot, writable)) in buffers.iter().enumerate() {
        let llvm_ty = llvm_type(k.ctx, &slot.ty)?;
        let (size, align) = layout(k.ctx, &slot.ty)?;
        params.push(ptr_type(dialect, &llvm_ty, 1));
        args.push(KernelArg::Buffer {
            name: var_name.clone(),
            llvm_ty,
            size,
            align,
            writable: *writable,
            space: 1,
        });
        k.vars.push((
            var_name.clone(),
            Slot {
                addr: format!("%buf{i}"),
                ty: slot.ty.clone(),
                space: 1,
            },
        ));
    }
    // The launch parameters never change, so on Apple GPUs they go in the
    // cached constant address space (2). PTX keeps them in global memory.
    let params_space = params_space(dialect);
    for (name, llvm_ty, writable, space) in [
        ("params", "i64", false, params_space),
        ("err", "i32", true, 1),
    ] {
        let (size, align) = (
            if llvm_ty == "i64" { 8 } else { 4 },
            if llvm_ty == "i64" { 8 } else { 4 },
        );
        params.push(ptr_type(dialect, llvm_ty, space));
        args.push(KernelArg::Buffer {
            name: name.into(),
            llvm_ty: llvm_ty.into(),
            size,
            align,
            writable,
            space,
        });
    }
    if dialect == Dialect::Air {
        params.push("i32".into());
        args.push(KernelArg::ThreadId);
    }

    let (lo, step, count, tid) = kernel_prologue(&mut k);
    k.emit(&format!("%in = icmp slt i64 {tid}, {count}"));
    k.emit("br i1 %in, label %work, label %exit");
    k.label("work");
    let offset = format!("%{}", k.fresh("t"));
    k.emit(&format!("{offset} = mul i64 {tid}, {step}"));
    let value = format!("%{}", k.fresh("t"));
    k.emit(&format!("{value} = add i64 {lo}, {offset}"));
    let iv = Slot {
        addr: k.alloca("i64"),
        ty: Type::Named("i64".into()),
        space: 0,
    };
    mem_store(&mut k, &iv, &value)?;
    k.vars.push((var.into(), iv));
    gen_block(&mut k, body, dir)?;
    k.emit("br label %exit");
    k.label("exit");

    let signature = params
        .iter()
        .enumerate()
        .map(|(i, ty)| match args.get(i) {
            Some(KernelArg::Buffer { name, .. }) if name == "params" => format!("{ty} %params"),
            Some(KernelArg::Buffer { name, .. }) if name == "err" => format!("{ty} %err"),
            Some(KernelArg::ThreadId) => format!("{ty} %tid"),
            _ => format!("{ty} %buf{i}"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let cc = if dialect == Dialect::Nvptx {
        "ptx_kernel "
    } else {
        ""
    };
    let text = format!(
        "define {cc}void @{name}({signature}) {{\nentry:\n{}{}  ret void\n}}\n",
        k.allocas, k.body
    );
    if dialect == Dialect::Air && text.contains("double") {
        return Err(CodegenError::Unsupported("f64 on the Apple GPU"));
    }
    parent.kernels.push(Kernel {
        dialect,
        name: name.clone(),
        text,
        param_types: params,
        args,
    });
    Ok(name)
}
