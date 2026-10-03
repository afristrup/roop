use crate::FnGen;

pub fn copy_bytes(g: &mut FnGen, dst: &str, src: &str, size: u64) {
    g.emit(&format!(
        "call void @llvm.memcpy.p0.p0.i64(ptr {dst}, ptr {src}, i64 {size}, i1 false)"
    ));
}
