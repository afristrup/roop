use crate::Kernel;

/// One NVPTX module holding every CUDA kernel.
pub fn ptx_module(types: &str, kernels: &[&Kernel]) -> String {
    let mut out = String::from(
        "target datalayout = \"e-i64:64-i128:128-v16:16-v32:32-n16:32:64\"\ntarget triple = \"nvptx64-nvidia-cuda\"\n\n",
    );
    out.push_str(types);
    out.push_str(
        "\ndeclare i32 @llvm.nvvm.read.ptx.sreg.tid.x()\ndeclare i32 @llvm.nvvm.read.ptx.sreg.ctaid.x()\ndeclare i32 @llvm.nvvm.read.ptx.sreg.ntid.x()\n\n",
    );
    for kernel in kernels {
        out.push_str(&kernel.text);
        out.push('\n');
    }
    out
}
