/// One kernel parameter. Buffers are device memory; the thread id is a
/// scalar the hardware supplies (AIR only, PTX reads special registers).
#[derive(Clone, Debug)]
pub enum KernelArg {
    Buffer {
        name: String,
        /// LLVM type of one element held by the buffer.
        llvm_ty: String,
        size: u64,
        align: u64,
        writable: bool,
    },
    ThreadId,
}
